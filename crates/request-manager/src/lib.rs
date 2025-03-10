/*
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use thiserror::Error;
use log::*;

pub use flowi_core::io::types::{FetchJob, Position, RequestId};
//use types::CacheEntry;
use flowi_core::io::priority::{PrioritizedRequest, PriorityInfo};
use flowi_core::io::cache::CacheStore;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid cache directory path")]
    InvalidPath,
    #[error("Failed to initialize cache: {0}")]
    InitializationError(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Configuration for the request manager
#[derive(Clone, Debug)]
pub struct RequestManagerConfig {
    pub cache_dir: PathBuf,
    pub min_delay: Duration,
    pub priority_weights: PriorityWeights,
}

/// Weights for different priority components
#[derive(Clone, Debug)]
pub struct PriorityWeights {
    pub selected: i32,
    pub visible: i32,
    pub cached: i32,
    pub distance_max: i32,
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            selected: 1000,
            visible: 100,
            cached: 50,
            distance_max: 80,
        }
    }
}

pub struct RequestManager {
    pub cache_store: CacheStore,
    request_queue: BinaryHeap<PrioritizedRequest>,
    pending_requests: HashMap<RequestId, PrioritizedRequest>,
    inflight_requests: HashSet<RequestId>,
    current_frame: u64,
    last_request: Instant,
    min_delay: Duration,
    selected_position: Option<Position>,
    priority_weights: PriorityWeights,
}

impl RequestManager {
    pub fn new(config: RequestManagerConfig) -> Result<Self> {
        let cache_store = CacheStore::new(&config.cache_dir)?;

        Ok(Self {
            cache_store,
            request_queue: BinaryHeap::new(),
            pending_requests: HashMap::new(),
            current_frame: 0,
            last_request: Instant::now() - Duration::from_secs(60), // Initialize to 1 minute ago
            min_delay: config.min_delay,
            selected_position: None,
            priority_weights: config.priority_weights,
            inflight_requests: HashSet::new(),
        })
    }

    // For testing: allows setting the last request time
    #[cfg(test)]
    pub fn set_last_request(&mut self, last_request: Instant) {
        self.last_request = last_request;
    }

    pub fn begin_frame(&mut self) {
        /*
        // Clear old requests that haven't been touched in the current frame
        self.pending_requests.retain(|_, req| {
            let retain = req.priority.frame_touched >= self.current_frame;
            if !retain {
                println!("Removing stale request: {}", req.url);
            }
            retain
        });

         */

        self.current_frame += 1;
    }

    pub fn set_selected_position(&mut self, pos: Option<Position>) {
        self.selected_position = pos;
        self.update_distance_scores();
    }

    pub fn request_data(
        &mut self,
        url: &str,
        id: RequestId,
        position: Position,
        is_visible: bool,
        is_selected: bool,
    ) -> Option<FetchJob> {
        // if this request is being processed we skip updating the queue
        if self.inflight_requests.contains(&id) {
            return None;
        }

        // If we have the url in the cache we return the job directly as it can
        // be processed immediately. We also add it to the inflight set to avoid
        // duplicate requests even if the caller shouldn't add it twice
        if let Some(cached) = self.cache_store.get_path(url).as_ref() {
            self.inflight_requests.insert(id);

            return Some(FetchJob::Cached {
                path: cached.clone(),
                id,
            });
        }

        let priority = PriorityInfo::new(
            self.current_frame,
            &self.priority_weights,
            is_visible,
            is_selected,
            position,
            self.selected_position,
        );

        let request = PrioritizedRequest::new(
            url,
            id,
            position,
            priority,
        );

        self.pending_requests.insert(id, request);

        None
    }

    pub fn process_frame(&mut self) -> Option<FetchJob> {
        // Move current frame requests to queue
        self.update_queue();

        // If queue is empty after update, nothing to do
        if self.request_queue.is_empty() {
            return None;
        }

        // Get the next request
        let next_request = self.request_queue.peek().unwrap().clone();

        // Check rate limiting
        let now = Instant::now();
        let time_since_last = now.duration_since(self.last_request);
        debug!("Time since last request: {:?} (min delay: {:?})",
                 time_since_last, self.min_delay);

        if time_since_last >= self.min_delay {
            debug!("Rate limit passed, processing request");
            // Remove from queues
            self.request_queue.pop();
            self.pending_requests.remove(&next_request.id);
            self.inflight_requests.insert(next_request.id);
            self.last_request = now;

            return Some(FetchJob::NeedsRequest {
                url: next_request.url,
                id: next_request.id,
            });
        }

        None
    }

    pub fn add_to_cache(&mut self, path: PathBuf) {
        self.cache_store.insert(path)
    }

    fn update_queue(&mut self) {
        // Clear the queue first to avoid duplicates
        self.request_queue.clear();

        // Add all requests from current frame
        for request in self.pending_requests.values() {
            if request.priority.frame_touched == self.current_frame {
                self.request_queue.push(request.clone());
            }
        }
    }

    fn update_distance_scores(&mut self) {
        if let Some(selected_pos) = self.selected_position {
            for request in self.pending_requests.values_mut() {
                request.priority.update_distance_score(
                    request.position,
                    selected_pos,
                    self.priority_weights.distance_max,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    fn setup_test_manager() -> (RequestManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = RequestManagerConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            min_delay: Duration::from_millis(500),
            priority_weights: PriorityWeights::default(),
        };
        let manager = RequestManager::new(config).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_basic_request_flow() {
        let (mut manager, _temp) = setup_test_manager();

        // Start frame
        manager.begin_frame();

        // Request some data
        manager.request_data(
            "test_url",
            1,
            Position { x: 0.0, y: 0.0 },
            true,
            false,
        );

        // Should get a request job since nothing is cached
        match manager.process_frame() {
            Some(FetchJob::NeedsRequest { url, id, .. }) => {
                assert_eq!(url, "test_url");
                assert_eq!(id, 1);
            }
            _ => panic!("Expected NeedsRequest"),
        }
    }

    #[test]
    fn test_priority_ordering() {
        let (mut manager, _temp) = setup_test_manager();
        manager.begin_frame();

        // Request items with different priorities
        manager.request_data(
            "selected.json",
            1,
            Position { x: 0.0, y: 0.0 },
            true,
            true,
        );

        manager.request_data(
            "visible.json",
            2,
            Position { x: 10.0, y: 10.0 },
            true,
            false,
        );

        manager.request_data(
            "hidden.json",
            3,
            Position { x: 20.0, y: 20.0 },
            false,
            false,
        );

        // Collect all items without rate limiting
        let mut results = Vec::new();
        while let Some(job) = manager.process_frame() {
            match job {
                FetchJob::NeedsRequest { url, .. } => {
                    results.push(url);
                    // Reset last_request to allow next request immediately
                    manager.set_last_request(Instant::now() - Duration::from_secs(60));
                }
                _ => panic!("Expected NeedsRequest"),
            }
        }

        // Verify order
        assert_eq!(results, vec![
            "selected.json".to_string(),
            "visible.json".to_string(),
            "hidden.json".to_string(),
        ]);
    }

    #[test]
    fn test_rate_limiting() {
        let (mut manager, _temp) = setup_test_manager();
        manager.begin_frame();

        let requests = vec![
            "url0.json",
            "url1.json",
            "url2.json",
        ];

        // Request multiple items
        for i in 0..3 {
            manager.request_data(
                &requests[i],
                i as _,
                Position { x: 0.0, y: 0.0 },
                true,
                false,
            );
        }

        // First request should go through
        assert!(matches!(manager.process_frame(), Some(FetchJob::NeedsRequest { .. })));

        // Second request should be rate limited
        assert!(manager.process_frame().is_none());
    }

    #[test]
    fn test_cache_hit() {
        let (mut manager, _temp_dir) = setup_test_manager();

        // Add item to cache and ensure it's properly inserted
        let url = "cached.json";
        let cache_path = manager.cache_store.get_path_for_url(url).to_owned();
        std::fs::write(&cache_path, "test content").unwrap(); // Create the actual file
        manager.add_to_cache(cache_path.clone());

        // Verify the item is in the cache
        assert!(manager.cache_store.contains_key(url), "Cache should contain the item");

        manager.begin_frame();
        let cached_request = manager.request_data(
            &url,
            1,
            Position { x: 0.0, y: 0.0 },
            true,
            false,
        );

        if let Some(FetchJob::Cached { path, .. }) = cached_request {
            assert_eq!(path, cache_path);
        } else {
            panic!("Expected cached response");
        }
    }

    #[test]
    fn test_distance_priority() {
        let (mut manager, _temp) = setup_test_manager();
        manager.begin_frame();

        // Set selected position
        let selected_pos = Position { x: 0.0, y: 0.0 };
        manager.set_selected_position(Some(selected_pos));

        // Request items at different distances
        manager.request_data(
            "near.json",
            1,
            Position { x: 10.0, y: 10.0 },
            true,
            false,
        );

        manager.request_data(
            "far.json",
            2,
            Position { x: 100.0, y: 100.0 },
            true,
            false,
        );

        // Near item should be processed first
        match manager.process_frame() {
            Some(FetchJob::NeedsRequest { url, .. }) => {
                assert_eq!(url, "near.json");
            }
            _ => panic!("Expected near item first"),
        }
    }
}

 */
