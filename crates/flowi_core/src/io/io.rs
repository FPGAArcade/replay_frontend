use crate::io::cache::CacheStore;
use crate::LoadOptions;
use job_system::JobSystem;
use job_system::{BoxAnySend, JobHandle, JobResult};
use log::{debug, error};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
pub struct IoHandle(pub u64);
pub type Callback = Box<dyn Fn(&[u8]) -> BoxAnySend + Send + 'static>;

pub struct IoSettings {
    pub cache_dir: String,
    pub remote_delay: Duration,
}

#[allow(dead_code)]
struct JobInfo {
    handle: JobHandle,
    url: String,
}

impl JobInfo {
    fn new(handle: JobHandle, url: &str) -> Self {
        Self {
            handle,
            url: url.to_owned(),
        }
    }
}

pub struct IoHandler {
    cache_store: CacheStore,
    settings: IoSettings,
    id_counter: u64,
    time: Instant,
    queue: VecDeque<(u64, Callback, String)>,
    inflight_jobs: HashMap<u64, JobInfo>,
    finished_jobs: HashMap<u64, BoxAnySend>,
}

pub enum LoadPriority {
    Low,
    Normal,
    High,
    Highest,
}

pub enum LoadState {
    NotStarted,
    Loading(f32),
    Loaded(BoxAnySend),
    Failed(String),
}

impl IoHandler {
    pub fn new(remote_delay: Duration) -> Self {
        let settings = IoSettings {
            cache_dir: CACHE_DIR.to_string(),
            remote_delay,
        };

        Self {
            cache_store: CacheStore::new(&settings.cache_dir).unwrap(),
            time: Instant::now() - Duration::from_secs(60),
            inflight_jobs: HashMap::with_capacity(256),
            finished_jobs: HashMap::with_capacity(256),
            queue: VecDeque::with_capacity(256),
            settings,
            id_counter: 1,
        }
    }

    pub fn load_with_callback(
        &mut self,
        url: &str,
        callback: Callback,
        _priority: LoadPriority,
        job_system: &JobSystem,
    ) -> IoHandle {
        let id = self.id_counter;
        self.id_counter += 1;

        // If data is in cache we can just start the job directly
        if self.cache_store.contains_key(url) {
            let t = Self::schedule_job_with_callback(job_system, url, DataSource::Cache, callback);
            self.inflight_jobs.insert(id, JobInfo::new(t, url));
        } else {
            self.queue.push_back((id, callback, url.to_owned()));
        }

        IoHandle(id)
    }

    pub fn load_image(
        &mut self,
        url: &str,
        image_options: LoadOptions,
        job_system: &JobSystem,
    ) -> IoHandle {
        // Create a callback that decodes the image with the given options
        let callback = Box::new(move |data: &[u8]| {
            crate::image::image_decoder::decode_zune(data, image_options)
        });

        // Use the generic load_with_callback function
        self.load_with_callback(url, callback, LoadPriority::Normal, job_system)
    }

    pub fn update(&mut self, job_system: &JobSystem) {
        // TODO: Prioritize jobs
        if self.queue.is_empty() {
            return;
        }

        if self.time.elapsed() > self.settings.remote_delay {
            self.time = Instant::now();
            if let Some(job) = self.queue.pop_front() {
                let t = Self::schedule_job_with_callback(
                    &job_system,
                    &job.2,
                    DataSource::Remote,
                    job.1,
                );
                self.inflight_jobs.insert(job.0, JobInfo::new(t, &job.2));
            }
        }
    }

    fn schedule_job_with_callback(
        jobs: &JobSystem,
        url: &str,
        ds: DataSource,
        callback: Callback,
    ) -> JobHandle {
        jobs.schedule_job(
            move |data: BoxAnySend| {
                read_data(
                    data,
                    ds,
                    Box::new(move |data| callback(data)),
                )
            },
            Box::new(url.to_string()),
        )
        .unwrap()
    }

    /// Get the load state of the handle and return the data if it is loaded. The user
    /// take ownership of the data. use get_loaded_as to get a reference to the data.
    pub fn return_loaded(&mut self, handle: IoHandle, _priority: LoadPriority) -> LoadState {
        if let Some(job_info) = self.inflight_jobs.get(&handle.0) {
            match job_info.handle.receiver.try_recv() {
                Ok(data) => {
                    //self.inflight_jobs.remove(&handle.0);
                    match data {
                        Ok(data) => LoadState::Loaded(data),
                        Err(e) => LoadState::Failed(format!("{}", e)),
                    }
                }
                // TODO: Handle more states here.
                _ => LoadState::Loading(0.0),
            }
        } else {
            LoadState::NotStarted
        }
    }

    pub fn get_loaded_as<T: 'static>(&mut self, handle: IoHandle) -> Option<&T> {
        if self.finished_jobs.contains_key(&handle.0) {
            self.finished_jobs.get(&handle.0)?.downcast_ref::<T>()
        } else {
            if let LoadState::Loaded(data) = self.return_loaded(handle, LoadPriority::Normal) {
                self.finished_jobs.insert(handle.0, data);
            }
            self.finished_jobs.get(&handle.0)?.downcast_ref::<T>()
        }
    }
}

/// Directory to store cached images
#[allow(dead_code)]
const CACHE_DIR: &str = "target/cache";

/// Writes the data to the cache directory
fn write_to_cache(url: &str, data: &[u8]) -> io::Result<()> {
    let mut cache_path = PathBuf::with_capacity(128);
    CacheStore::get_cache_path(url, CACHE_DIR, &mut cache_path);

    debug!("Start write to cache: {} -> {:?}", url, cache_path);

    // Write the image to the cache directory
    use std::io::Write;
    let mut file = File::create(&cache_path)?;
    file.write_all(data)?;

    debug!("Done  write to cache: {} -> {:?}", url, cache_path);
    Ok(())
}

/// Fetches data from the remote URL.
///
/// This will also write data ta the cache. Even if we fail to write to the cache we return the data
/// and us it anyway. If the disk might be full, bad, or something we still want to continue
/// the progress. We will log an error in the log that something went wrong so the user
/// can know about it.
#[allow(dead_code)]
pub fn read_data_from_remote(url: &str) -> io::Result<Vec<u8>> {
    // Fetch the image from the URL
    debug!("Start read from remote: {}", url);

    let resp = ureq::get(url)
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut reader = resp.into_body().into_with_config().reader();

    use std::io::Read;
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    match write_to_cache(url, &bytes) {
        Ok(_) => Ok(bytes),
        Err(e) => {
            error!(
                "Failed to write to cache: {} but trying to use data anyway",
                e
            );
            Ok(bytes)
        }
    }
}

/// Read the data from the cache.
#[allow(dead_code)]
fn read_data_from_cache(url: &str) -> io::Result<Vec<u8>> {
    let mut cache_path = PathBuf::with_capacity(128);
    CacheStore::get_cache_path(url, CACHE_DIR, &mut cache_path);

    debug!("Start read from cache: {} -> {:?}", url, cache_path);

    use std::io::Read;
    let mut file = File::open(&cache_path)?;
    let mut contents = Vec::new();
    let size = file.read_to_end(&mut contents)?;

    debug!(
         "Done  read from cache: {} -> {:?} (size {})",
        url, cache_path, size
    );

    Ok(contents)
}

// First, let's create an enum to handle the data source
#[derive(Debug, PartialEq)]
enum DataSource {
    Cache,
    Remote,
}

type BinaryCallback = Box<dyn Fn(&[u8]) -> BoxAnySend>;

// And an enum for how to handle the raw data

fn read_data(data: BoxAnySend, source: DataSource, callback: BinaryCallback) -> JobResult<BoxAnySend> {
    let url = data.downcast::<String>().unwrap();

    let data = match source {
        DataSource::Cache => read_data_from_cache(&url)?,
        DataSource::Remote => read_data_from_remote(&url)?,
    };

    Ok(callback(&data))

        /*
    let result = match format {
        DataFormat::String(callback) => {
            let string = std::str::from_utf8(&data).expect("Failed to convert to string");
            callback(string)
        }
        DataFormat::Binary(callback) => callback(&data),
    };

         */

    //Ok(result)
}
