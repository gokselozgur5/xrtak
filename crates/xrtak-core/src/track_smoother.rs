//! Velocity and heading filter smoother for tactical entity tracks.

use serde::{Deserialize, Serialize};

/// Normalizes heading in degrees to [0.0, 360.0).
pub fn wrap_heading_deg(deg: f64) -> f64 {
    (deg % 360.0 + 360.0) % 360.0
}

/// Computes signed shortest angular difference from -> to in [-180.0, 180.0).
pub fn heading_delta_deg(from_deg: f64, to_deg: f64) -> f64 {
    let mut diff = (to_deg - from_deg) % 360.0;
    if diff >= 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    diff
}
