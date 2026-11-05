use crate::TacticalEntity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Single historical keyframe captured by `AarTimeline` (`REQ-AAR-01`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AarFrame {
    pub sim_time_sec: f64,
    pub unix_ms: u64,
    pub entities: Vec<TacticalEntity>,
}

/// Metadata summary of the active 4D AAR ring-buffer (`REQ-AAR-02`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AarSummary {
    pub frame_count: usize,
    pub max_frames: usize,
    pub min_time_sec: f64,
    pub max_time_sec: f64,
}

/// Bounded ring-buffer 4D After-Action Review (AAR) recorder & temporal interpolator (`REQ-AAR-01..03`).
#[derive(Debug, Clone)]
pub struct AarTimeline {
    pub max_frames: usize,
    pub min_capture_interval_sec: f64,
    pub frames: VecDeque<AarFrame>,
}

impl Default for AarTimeline {
    fn default() -> Self {
        Self::new(720, 0.5)
    }
}

impl AarTimeline {
    pub fn new(max_frames: usize, min_capture_interval_sec: f64) -> Self {
        Self {
            max_frames: max_frames.max(8),
            min_capture_interval_sec: min_capture_interval_sec.max(0.05),
            frames: VecDeque::with_capacity(max_frames.max(8)),
        }
    }

    /// Record a keyframe if at least `min_capture_interval_sec` has elapsed since the last frame.
    pub fn maybe_record(
        &mut self,
        sim_time_sec: f64,
        unix_ms: u64,
        entities_map: &HashMap<String, TacticalEntity>,
    ) -> bool {
        if let Some(last) = self.frames.back() {
            if (sim_time_sec - last.sim_time_sec) < self.min_capture_interval_sec {
                return false;
            }
        }
        let mut entities: Vec<TacticalEntity> = entities_map.values().cloned().collect();
        entities.sort_by(|a, b| a.uid.cmp(&b.uid));
        if self.frames.len() >= self.max_frames {
            self.frames.pop_front();
        }
        self.frames.push_back(AarFrame { sim_time_sec, unix_ms, entities });
        true
    }

    pub fn summary(&self) -> AarSummary {
        AarSummary {
            frame_count: self.frames.len(),
            max_frames: self.max_frames,
            min_time_sec: self.frames.front().map(|f| f.sim_time_sec).unwrap_or(0.0),
            max_time_sec: self.frames.back().map(|f| f.sim_time_sec).unwrap_or(0.0),
        }
    }
}
