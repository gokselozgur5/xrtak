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
