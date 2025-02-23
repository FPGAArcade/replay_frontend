// priority.rs
use std::cmp::Ordering;
use std::time::Instant;
use request_manager::types::{Position, RequestId};
use request_manager::PriorityWeights;

#[derive(Debug, Clone)]
pub struct PriorityInfo {
    pub base_score: i32,
    pub visibility_score: i32,
    pub selection_score: i32,
    pub distance_score: i32,
    pub frame_touched: u64,
}

#[derive(Debug, Clone)]
pub struct PrioritizedRequest {
    // TODO: String allocator
    pub url: String,
    pub id: RequestId,
    pub position: Position,
    pub priority: PriorityInfo,
    pub queue_time: Instant,
}

impl PrioritizedRequest {
    pub fn new(
        url: &str,
        id: RequestId,
        position: Position,
        priority: PriorityInfo,
    ) -> Self {
        Self {
            url: url.to_string(),
            id,
            position,
            priority,
            queue_time: Instant::now(),
        }
    }
}

impl Ord for PrioritizedRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare total scores
        self.priority.total_score().cmp(&other.priority.total_score())
            // Then by frame touched (more recent = higher priority)
            .then(self.priority.frame_touched.cmp(&other.priority.frame_touched))
            // Finally by queue time (earlier = higher priority)
            .then(other.queue_time.cmp(&self.queue_time))
    }
}

impl PartialOrd for PrioritizedRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PrioritizedRequest {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for PrioritizedRequest {}

impl PriorityInfo {
    pub fn new(
        frame: u64,
        weights: &PriorityWeights,
        is_visible: bool,
        is_selected: bool,
        position: Position,
        selected_position: Option<Position>,
    ) -> Self {
        let mut info = Self {
            base_score: 0,
            visibility_score: if is_visible { weights.visible } else { 0 },
            selection_score: if is_selected { weights.selected } else { 0 },
            distance_score: 0,
            frame_touched: frame,
        };

        if let Some(selected_pos) = selected_position {
            info.update_distance_score(position, selected_pos, weights.distance_max);
        }

        info
    }

    pub fn update_distance_score(&mut self, pos: Position, selected_pos: Position, max_score: i32) {
        let distance = pos.distance_to(&selected_pos);
        let max_distance = 1000.0; // Adjust based on UI scale
        let normalized_distance = (max_distance - distance.min(max_distance)) / max_distance;
        self.distance_score = (normalized_distance * max_score as f32) as i32;
    }

    pub fn total_score(&self) -> i32 {
        self.base_score +
            self.visibility_score +
            self.selection_score +
            self.distance_score
    }
}