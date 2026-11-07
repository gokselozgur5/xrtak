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
