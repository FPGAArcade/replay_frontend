// types.rs
use std::path::PathBuf;
use std::time::Instant;

pub type RequestId = u64;

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub path: PathBuf,
    //pub fetched_at: Instant,
}

#[derive(Debug)]
pub enum FetchJob {
    Cached {
        url: String,
        path: PathBuf,
        id: RequestId,
    },
    NeedsRequest {
        url: String,
        id: RequestId,
        execute_after: Instant,
    },
}