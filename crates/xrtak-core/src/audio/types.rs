use serde::{Deserialize, Serialize};

/// 3D position vector in audio space (meters).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
