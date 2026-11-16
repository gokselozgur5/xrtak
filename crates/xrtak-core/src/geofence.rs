//! Geofence polygon ray-casting containment checker.

use serde::{Deserialize, Serialize};

/// Point coordinate for geofencing calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeofencePoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}
