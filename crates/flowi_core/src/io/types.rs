// types.rs
use std::path::PathBuf;

pub type RequestId = u64;

#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Debug)]
pub enum FetchJob {
    Cached { path: PathBuf, id: RequestId },
    NeedsRequest { url: String, id: RequestId },
}
