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

impl PoseOrientation {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }
}

    pub fn normalized(&self) -> Self {
        let n = self.norm();
        if n < 1e-12 {
            Self::IDENTITY
        } else {
            Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
                w: self.w / n,
            }
        }
    }
