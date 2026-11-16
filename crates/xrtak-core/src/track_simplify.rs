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

/// Great-circle Haversine distance between two WGS84 coordinates in meters.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + r_lat1.cos() * r_lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    WGS84_EARTH_RADIUS_M * c
}

/// Initial true bearing from coordinate 1 to coordinate 2 in degrees [0, 360).
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let y = d_lon.sin() * r_lat2.cos();
    let x = r_lat1.cos() * r_lat2.sin() - r_lat1.sin() * r_lat2.cos() * d_lon.cos();
    let brg = y.atan2(x).to_degrees();
    (brg + 360.0) % 360.0
}

/// Computes perpendicular distance from point P to line segment A->B in meters.
pub fn perpendicular_distance_to_segment_m(
    p: &TrackPoint,
    a: &TrackPoint,
    b: &TrackPoint,
    include_elevation: bool,
) -> f64 {
    let seg_dist = haversine_distance_m(a.lat, a.lon, b.lat, b.lon);
    if seg_dist < 1e-6 {
        let d = haversine_distance_m(p.lat, p.lon, a.lat, a.lon);
        return if include_elevation {
            (d.powi(2) + (p.hae_m - a.hae_m).powi(2)).sqrt()
        } else {
            d
        };
    }

    let d_ap = haversine_distance_m(a.lat, a.lon, p.lat, p.lon);
    let brg_ab = initial_bearing_deg(a.lat, a.lon, b.lat, b.lon).to_radians();
    let brg_ap = initial_bearing_deg(a.lat, a.lon, p.lat, p.lon).to_radians();
    let delta_brg = brg_ap - brg_ab;

    let along_dist = d_ap * delta_brg.cos();
    if along_dist <= 0.0 {
        let d = haversine_distance_m(p.lat, p.lon, a.lat, a.lon);
        return if include_elevation {
            (d.powi(2) + (p.hae_m - a.hae_m).powi(2)).sqrt()
        } else {
            d
        };
    }

    if along_dist >= seg_dist {
        let d = haversine_distance_m(p.lat, p.lon, b.lat, b.lon);
        return if include_elevation {
            (d.powi(2) + (p.hae_m - b.hae_m).powi(2)).sqrt()
        } else {
            d
        };
    }

    let cross_dist = (d_ap * delta_brg.sin()).abs();
    if include_elevation {
        let t = along_dist / seg_dist;
        let interp_hae = a.hae_m + t * (b.hae_m - a.hae_m);
        let dz = p.hae_m - interp_hae;
        (cross_dist.powi(2) + dz.powi(2)).sqrt()
    } else {
        cross_dist
    }
}

fn rdp_recursive(
    points: &[TrackPoint],
    epsilon_m: f64,
    include_elevation: bool,
    max_err: &mut f64,
) -> Vec<TrackPoint> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let first = &points[0];
    let last = &points[points.len() - 1];

    let mut max_dist = 0.0;
    let mut split_idx = 0usize;

    for (i, pt) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let dist = perpendicular_distance_to_segment_m(pt, first, last, include_elevation);
        if dist > max_dist {
            max_dist = dist;
            split_idx = i;
        }
    }
