//! Dead-Reckoning, Kinematic Track Extrapolation & CEP Uncertainty Ring Engine.
//!
//! Complies with REQ-TRK-01..02 and ICD-20 for stale/coasting tactical tracks.

use serde::{Deserialize, Serialize};

/// Projected kinematic waypoint at lookahead delta time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPredictionPoint {
    pub dt_sec: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
    pub cep_radius_m: f64,
}

/// Predicted trajectory and confidence boundary for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPrediction {
    pub uid: String,
    pub callsign: String,
    pub current_enu: (f64, f64, f64),
    pub speed_mps: f64,
    pub course_deg: f64,
    pub is_stale: bool,
    pub points: Vec<TrackPredictionPoint>,
}

/// Request to project tracks over a set of lookahead horizons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPredictRequest {
    pub horizons_sec: Option<Vec<f64>>,
    pub stale_threshold_sec: Option<f64>,
}

/// Extrapolates 3D ENU position after dt_sec given velocity vector and heading.
pub fn compute_kinematic_extrapolation(
    current_enu: (f64, f64, f64),
    speed_mps: f64,
    course_deg: f64,
    vert_speed_mps: f64,
    dt_sec: f64,
) -> (f64, f64, f64) {
    let rad = course_deg.to_radians();
    let dx = speed_mps * rad.sin() * dt_sec;
    let dy = speed_mps * rad.cos() * dt_sec;
    let dz = vert_speed_mps * dt_sec;
    (
        current_enu.0 + dx,
        current_enu.1 + dy,
        current_enu.2 + dz,
    )
}

/// Computes expanding Circular Error Probable (CEP) confidence radius in meters.
pub fn compute_cep_radius_m(initial_ce_m: f64, speed_mps: f64, dt_sec: f64) -> f64 {
    let r0 = initial_ce_m.max(2.0);
    let sigma_v = 0.5; // velocity uncertainty m/s
    let sigma_omega = 0.03; // turn rate standard deviation rad/s
    let vel_variance = (sigma_v * dt_sec).powi(2);
    let turn_variance = 0.25 * (sigma_omega * speed_mps * dt_sec.powi(2)).powi(2);
    (r0.powi(2) + vel_variance + turn_variance).sqrt()
}
