//! Tactical route navigation waypoint sequence generator and ETA estimator.

use serde::{Deserialize, Serialize};

/// Earth radius in meters for geodetic calculations.
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculate Haversine surface distance in meters between two coordinates.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let (r1, r2) = (lat1.to_radians(), lat2.to_radians());
    let a = (d_lat / 2.0).sin().powi(2) + r1.cos() * r2.cos() * (d_lon / 2.0).sin().powi(2);
    EARTH_RADIUS_M * 2.0 * a.sqrt().asin()
}

/// Calculate initial forward azimuth / bearing in degrees [0, 360).
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (r1, r2) = (lat1.to_radians(), lat2.to_radians());
    let d_lon = (lon2 - lon1).to_radians();
    let y = d_lon.sin() * r2.cos();
    let x = r1.cos() * r2.sin() - r1.sin() * r2.cos() * d_lon.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

/// 3D distance accounting for elevation delta in meters.
pub fn slant_distance_3d_m(
    lat1: f64,
    lon1: f64,
    alt1: f64,
    lat2: f64,
    lon2: f64,
    alt2: f64,
) -> f64 {
    let surface_d = haversine_distance_m(lat1, lon1, lat2, lon2);
    let dz = alt2 - alt1;
    (surface_d.powi(2) + dz.powi(2)).sqrt()
}

/// Cross-track error in meters from point P to segment A->B.
/// Positive is right of track; negative is left of track.
pub fn cross_track_error_m(
    plat: f64,
    plon: f64,
    alat: f64,
    alon: f64,
    blat: f64,
    blon: f64,
) -> f64 {
    let d13 = haversine_distance_m(alat, alon, plat, plon) / EARTH_RADIUS_M;
    let brg13 = initial_bearing_deg(alat, alon, plat, plon).to_radians();
    let brg12 = initial_bearing_deg(alat, alon, blat, blon).to_radians();
    let xtd = (d13.sin() * (brg13 - brg12).sin()).asin();
    xtd * EARTH_RADIUS_M
}

/// Tactical waypoint classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaypointType {
    Start,
    TurnPoint,
    RallyPoint,
    Target,
    Checkpoint,
    Ingress,
    Egress,
    End,
}

/// Single navigation waypoint with 3D coordinates and constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavWaypoint {
    pub id: String,
    pub callsign: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
    pub wp_type: WaypointType,
    pub speed_limit_mps: Option<f64>,
}

impl NavWaypoint {
    /// Create a new tactical navigation waypoint.
    pub fn new(
        id: impl Into<String>,
        callsign: impl Into<String>,
        lat: f64,
        lon: f64,
        alt_m: f64,
        wp_type: WaypointType,
    ) -> Self {
        Self {
            id: id.into(),
            callsign: callsign.into(),
            lat,
            lon,
            alt_m,
            wp_type,
            speed_limit_mps: None,
        }
    }
}
