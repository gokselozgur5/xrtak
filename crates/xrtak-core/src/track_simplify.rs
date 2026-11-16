//! CoT track history polyline simplification using Ramer-Douglas-Peucker (RDP).
//!
//! Complies with REQ-TRK-03 and ICD-20 for track history compression and ATAK breadcrumbs.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

/// WGS84 mean earth radius in meters.
pub const WGS84_EARTH_RADIUS_M: f64 = 6_371_000.0;

/// WGS84 kinematic position point along a tactical entity track history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub timestamp_ms: Option<u64>,
    pub speed_mps: Option<f64>,
    pub course_deg: Option<f64>,
}

impl TrackPoint {
    pub fn new(lat: f64, lon: f64, hae_m: f64) -> Self {
        Self {
            lat,
            lon,
            hae_m,
            timestamp_ms: None,
            speed_mps: None,
            course_deg: None,
        }
    }

    pub fn with_time(mut self, timestamp_ms: u64) -> Self {
        self.timestamp_ms = Some(timestamp_ms);
        self
    }

    pub fn with_kinematics(mut self, speed_mps: f64, course_deg: f64) -> Self {
        self.speed_mps = Some(speed_mps);
        self.course_deg = Some(course_deg);
        self
    }
}

/// Configuration parameters for track polyline simplification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSimplifyConfig {
    pub epsilon_m: f64,
    pub include_elevation: bool,
    pub max_time_gap_sec: Option<f64>,
}

impl Default for TrackSimplifyConfig {
    fn default() -> Self {
        Self {
            epsilon_m: 5.0,
            include_elevation: false,
            max_time_gap_sec: None,
        }
    }
}

/// Result of track polyline simplification including compression statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSimplifyResult {
    pub points: Vec<TrackPoint>,
    pub original_count: usize,
    pub simplified_count: usize,
    pub reduction_ratio: f64,
    pub max_error_m: f64,
}
