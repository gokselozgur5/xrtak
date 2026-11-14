use serde::{Deserialize, Serialize};

/// Spatialized binaural channel gains (left, right) in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelGains {
    pub left: f64,
    pub right: f64,
}
