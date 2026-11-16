//! Geofence polygon ray-casting containment checker.

use serde::{Deserialize, Serialize};

/// Point coordinate for geofencing calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeofencePoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}

impl GeofencePoint {
    pub fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: None }
    }

    pub fn new_3d(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z: Some(z) }
    }
}

/// 2D axis-aligned bounding box for fast containment pre-filtering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
