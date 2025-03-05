use priority_queue::PriorityQueue;
use crate::{io::cache::CacheStore, LoadOptions};
use job_system::{JobSystem, BoxAnySend, JobHandle, JobResult};
use log::{debug, error, info, warn};
use std::{
    collections::HashMap,
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
    io,
};

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

struct QueueItem {
    callback: Callback,
    url: String,
    priority: LoadPriority,
}

impl QueueItem {
    fn new(callback: Callback, url: String, priority: LoadPriority) -> Self {
        Self {
            callback,
            url,
            priority,
        }
    }
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for QueueItem {}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

pub struct IoHandler {
    cache_store: CacheStore,
    settings: IoSettings,
    id_counter: u64,
    time: Instant,
    queue: PriorityQueue<u64, QueueItem>,
    inflight_jobs: HashMap<u64, JobInfo>,
    finished_jobs: HashMap<u64, BoxAnySend>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Highest = 3,
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
            queue: PriorityQueue::new(),
            settings,
            id_counter: 1,
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

        // If data is in cache we can just start the job directly
        if self.cache_store.contains_key(url) {
            let t = Self::schedule_job_with_callback(job_system, url, DataSource::Cache, callback);
            self.inflight_jobs.insert(id, JobInfo::new(t, url));
        } else {
            self.queue.push(id, QueueItem::new(callback, url.to_owned(), priority));
        }

        IoHandle(id)
    }

    pub fn load_image(
        &mut self,
        url: &str,
        image_options: LoadOptions,
        job_system: &JobSystem,
    ) -> IoHandle {
        info!("Load image: {}", url);

        if url.ends_with(".gif") {
            warn!("GIF images are not supported yet: {}", url);
            return IoHandle(0);
        }

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
            if let Some((id, job)) = self.queue.pop() {
                let t = Self::schedule_job_with_callback(
                    &job_system,
                    &job.url,
                    DataSource::Remote,
                    job.callback,
                );
                self.inflight_jobs.insert(id, JobInfo::new(t, &job.url));
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

    /// Hint the priority of the handle. This is useful for example if we want to load
    /// a low priority image in the background. The code may not extract the data directly
    /// but if it needs something to be visible it can hint the priority to load the data.
    pub fn hint_priority(&mut self, handle: IoHandle, priority: LoadPriority) {
        self.queue.change_priority_by(&handle.0, |item| {
            item.priority = priority;
        });
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
    info!("Start read from remote: {}", url);

    let resp = ureq::get(url)
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut reader = resp.into_body().into_with_config().reader();

    use std::io::Read;
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    info!("Done  read from remote: {} (size {})", url, bytes.len());

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
