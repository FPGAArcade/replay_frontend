use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use job_system::{JobHandle, JobResult, BoxAnySend};
use crate::io::cache::CacheStore;
use crate::io::io::LoadState::Loaded;
use log::{debug, error};

#[derive(Debug, Copy, Clone)]
pub struct IoHandle(pub u64);
pub type Callback<T> = Box<dyn Fn(&[u8]) -> T + Send + 'static>;

pub struct IoSettings {
    pub cache_dir: String,
    pub remote_delay: Duration,
}

struct JobInfo {
    handle: JobHandle,
    url: String,
}

pub struct IoHandler {
    cache_store: CacheStore,
    settings: IoSettings,
    id_counter: u64,
    jobs: HashMap<u64, JobInfo>,
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
    Failed(String)
}

impl IoHandler {
    pub fn new(remote_delay: Duration) -> Self {
        let settings = IoSettings {
            cache_dir: CACHE_DIR.to_string(),
            remote_delay,
        };

        Self {
            cache_store: CacheStore::new(&settings.cache_dir).unwrap(),
            jobs: HashMap::with_capacity(256),
            finished_jobs: HashMap::with_capacity(256),
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

    pub fn load_with_callback<T>(&mut self, url: &str, callback: Callback<T>) -> IoHandle {
        // Implementation here
        IoHandle(0)
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
        if let Some(job_info) = self.jobs.get(&handle.0) {
            match job_info.handle.receiver.try_recv() {
                Ok(data) => {
                    self.jobs.remove(&handle.0);
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

    let mut reader = resp
        .into_body()
        .into_with_config()
        .reader();

    use std::io::Read;
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    match write_to_cache(url, &bytes) {
        Ok(_) =>  Ok(bytes),
        Err(e) => {
            error!("Failed to write to cache: {} but trying to use data anyway", e);
            Ok(bytes)
        }
    }
}

/// Read the data from the cache.
#[allow(dead_code)]
fn read_data_from_cache<T> (url: &str) -> io::Result<Vec<u8>> {
    let mut cache_path = PathBuf::with_capacity(128);
    CacheStore::get_cache_path(url, CACHE_DIR, &mut cache_path);

    debug!("Start read from cache: {} -> {:?}", url, cache_path);

    use std::io::Read;
    let mut file = File::open(&cache_path)?;
    let mut contents = Vec::new();
    let size = file.read_to_end(&mut contents)?;

    debug!("Start read from cache: {} -> {:?} (size {})", url, cache_path, size);

    Ok(contents)
}

// First, let's create an enum to handle the data source
enum DataSource {
    Cache,
    Remote,
}

// And an enum for how to handle the raw data
enum DataFormat<T> {
    String(Box<dyn Fn(&str) -> T + Send>),
    Binary(Box<dyn Fn(&[u8]) -> T + Send>),
}

fn read_data<T>(data: BoxAnySend, source: DataSource, format: DataFormat<T>) -> JobResult<BoxAnySend>
where
    T: Sized + Send + 'static,
{
    let url = data.downcast::<String>().unwrap();

    let data = match source {
        DataSource::Cache => read_data_from_cache::<T>(&url)?,
        DataSource::Remote => read_data_from_remote(&url)?,
    };

    let result = match format {
        DataFormat::String(callback) => {
            let string = std::str::from_utf8(&data).expect("Failed to convert to string");
            callback(string)
        },
        DataFormat::Binary(callback) => callback(&data),
    };

    Ok(Box::new(result) as BoxAnySend)
}
