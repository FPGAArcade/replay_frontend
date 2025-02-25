use crate::io::cache::CacheStore;
use crate::io::io::LoadState::Loaded;
use crate::LoadOptions;
use job_system::JobSystem;
use job_system::{BoxAnySend, JobHandle, JobResult};
use log::{debug, error};
use std::collections::HashMap;
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

struct JobInfo {
    handle: JobHandle,
    url: String,
}

impl JobInfo {
    fn new(handle: JobHandle, url: &str) -> Self {
        Self { handle, url: url.to_owned() }
    }
}

pub struct IoHandler {
    cache_store: CacheStore,
    settings: IoSettings,
    id_counter: u64,
    time: Instant,
    queue: HashMap<u64, Callback>,
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
            queue: HashMap::with_capacity(256),
            settings,
            id_counter: 1,
        }
    }

    pub fn load(&mut self, url: &str) -> IoHandle {
        if self.cache_store.contains_key(url) {
            self.load_from_cache(url)
        } else {
            self.load_from_remote(url)
        }
    }

    fn load_from_cache(&self, url: &str) -> IoHandle {
        IoHandle(0) // Placeholder return value
    }

    fn load_from_remote(&self, url: &str) -> IoHandle {
        IoHandle(0) // Placeholder return value
    }

    /// Get the load state of the handle and return the data if it is loaded. The user
    /// take ownership of the data. use get_loaded_as to get a reference to the data.
    pub fn return_loaded(&mut self, handle: IoHandle, _priority: LoadPriority) -> LoadState {
        if let Some(job_info) = self.inflight_jobs.get(&handle.0) {
            match job_info.handle.receiver.try_recv() {
                Ok(data) => {
                    self.inflight_jobs.remove(&handle.0);
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
            self.get_loaded(handle)
        } else {
            self.load_and_store(handle);
            self.get_loaded(handle)
        }
    }

    fn get_loaded<T: 'static>(&self, handle: IoHandle) -> Option<&T> {
        self.finished_jobs.get(&handle.0)?.downcast_ref::<T>()
    }

    fn load_and_store(&mut self, handle: IoHandle) {
        if let LoadState::Loaded(data) = self.return_loaded(handle, LoadPriority::Normal) {
            self.finished_jobs.insert(handle.0, data);
        }
    }

    pub fn load_with_callback(
        &mut self,
        url: &str,
        callback: Callback,
        priority: LoadPriority,
        job_system: &JobSystem,
    ) -> IoHandle {
        let id = self.id_counter;
        self.id_counter += 1;

        let data_source = if self.cache_store.contains_key(url) {
            DataSource::Cache
        } else {
            DataSource::Remote
        };

        // If data is in cache we can just start the job directly
        if data_source == DataSource::Cache {
            let t = job_system
                .schedule_job(
                    move |data: BoxAnySend| {
                        read_data(
                            data,
                            data_source,
                            DataFormat::Binary(Box::new(move |data| callback(data)))
                        )
                    },
                    Box::new(url.to_string()),
                )
                .unwrap();

            self.inflight_jobs.insert(id, JobInfo::new(t, url));
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
    /*
    pub fn get_loaded_as<T: 'static>(&mut self, handle: IoHandle) -> Option<&T> {
        // first check if is in the loaded list
        if let Some(job_info) = self.finished_jobs.get(&handle.0) {
            job_info.downcast_ref::<T>()
        } else {
            let data = self.return_loaded(handle, Priority::Normal);

            match data {
                LoadState::Loaded(data) => {
                    let output = data.downcast_ref::<T>();
                    self.finished_jobs.insert(handle.0, data);
                    output
                }
                _ => None,
            }
        }
    }

     */
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
        "Start read from cache: {} -> {:?} (size {})",
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

type StringCallback = Box<dyn Fn(&str) -> BoxAnySend>;
type BinaryCallback = Box<dyn Fn(&[u8]) -> BoxAnySend>;

// And an enum for how to handle the raw data
enum DataFormat {
    String(StringCallback),
    Binary(BinaryCallback),
}

fn read_data(
    data: BoxAnySend,
    source: DataSource,
    format: DataFormat,
) -> JobResult<BoxAnySend>
{
    let url = data.downcast::<String>().unwrap();

    let data = match source {
        DataSource::Cache => read_data_from_cache(&url)?,
        DataSource::Remote => read_data_from_remote(&url)?,
    };

    let result = match format {
        DataFormat::String(callback) => {
            let string = std::str::from_utf8(&data).expect("Failed to convert to string");
            callback(string)
        }
        DataFormat::Binary(callback) => callback(&data),
    };

    Ok(Box::new(result) as BoxAnySend)
}
