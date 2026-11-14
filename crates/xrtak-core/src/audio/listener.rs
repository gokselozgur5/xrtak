use super::types::AudioVec3;
use serde::{Deserialize, Serialize};

/// Listener head pose orientation in radians (yaw, pitch, roll).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ListenerOrientation {
    pub yaw_rad: f64,
    pub pitch_rad: f64,
    pub roll_rad: f64,
}

impl Default for ListenerOrientation {
    fn default() -> Self {
        Self {
            yaw_rad: 0.0,
            pitch_rad: 0.0,
            roll_rad: 0.0,
        }
    }
}

/// Listener pose representation including position and head orientation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ListenerPose {
    pub position: AudioVec3,
    pub orientation: ListenerOrientation,
}
