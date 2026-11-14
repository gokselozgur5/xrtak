use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PosePosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PosePosition {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_tuple(self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoseOrientation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}
