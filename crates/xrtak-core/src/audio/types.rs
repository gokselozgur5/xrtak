use serde::{Deserialize, Serialize};

/// 3D position vector in audio space (meters).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl AudioVec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}
