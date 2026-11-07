// crates/xrtak-core/src/battery.rs
//! UAV Wind-Corrected Bingo Battery & Asymmetric RTH Polygon Engine (ICD-26, REQ-BAT-01..02).

use serde::{Deserialize, Serialize};

/// Ambient wind vector in local ENU frame.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindVector {
    /// Wind blowing towards East (m/s)
    pub east_mps: f64,
    /// Wind blowing towards North (m/s)
    pub north_mps: f64,
}

impl Default for WindVector {
    fn default() -> Self {
        Self { east_mps: 0.0, north_mps: 0.0 }
    }
}

/// UAV Battery state and electrical parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UavBatteryState {
    pub callsign: String,
    pub battery_remaining_pct: f64,
    pub battery_capacity_wh: f64,
    pub current_power_draw_w: f64,
    pub reserve_margin_pct: f64,
    pub cruise_airspeed_mps: f64,
    pub current_east_m: f64,
    pub current_north_m: f64,
    pub current_up_m: f64,
    pub home_east_m: f64,
    pub home_north_m: f64,
    pub home_up_m: f64,
}

/// 2D Point on RTH footprint perimeter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RthPolygonVertex {
    pub bearing_deg: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub max_radius_m: f64,
}
