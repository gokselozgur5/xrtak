//! KML document exporter for CoT event tracks.

use serde::{Deserialize, Serialize};

/// WGS84 kinematic position point along a tactical track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KmlTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub timestamp_ms: Option<u64>,
    pub speed_mps: Option<f64>,
    pub course_deg: Option<f64>,
}
