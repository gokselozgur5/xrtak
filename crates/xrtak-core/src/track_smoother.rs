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

/// Smooths heading using shortest angular path across 0/360 boundary.
pub fn smooth_heading_deg(prev_deg: f64, new_deg: f64, alpha: f64) -> f64 {
    let a = alpha.clamp(0.0, 1.0);
    let delta = heading_delta_deg(prev_deg, new_deg);
    wrap_heading_deg(prev_deg + a * delta)
}

/// Computes scalar exponential moving average with clamping.
pub fn smooth_scalar(prev: f64, new_val: f64, alpha: f64) -> f64 {
    let a = alpha.clamp(0.0, 1.0);
    prev + a * (new_val - prev)
}
