use serde::{Deserialize, Serialize};

/// Spatialized binaural channel gains (left, right) in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelGains {
    pub left: f64,
    pub right: f64,
}

impl ChannelGains {
    pub const fn new(left: f64, right: f64) -> Self {
        Self { left, right }
    }

    pub fn clamp(&self) -> Self {
        Self {
            left: self.left.clamp(0.0, 1.0),
            right: self.right.clamp(0.0, 1.0),
        }
    }
}

/// Result of evaluating spatial audio emitter relative to listener.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatializedAudio {
    pub emitter_id: String,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub distance_m: f64,
    pub gains: ChannelGains,
    pub delay_ms: f64,
}
