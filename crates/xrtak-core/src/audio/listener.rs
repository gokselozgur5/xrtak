use serde::{Deserialize, Serialize};

/// Listener head pose orientation in radians (yaw, pitch, roll).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ListenerOrientation {
    pub yaw_rad: f64,
    pub pitch_rad: f64,
    pub roll_rad: f64,
}
