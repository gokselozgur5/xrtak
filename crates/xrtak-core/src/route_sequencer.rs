//! CoT route plan waypoint sequencer and ETA estimator.

use serde::{Deserialize, Serialize};

const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Great-circle Haversine distance between two WGS84 positions in meters.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + r_lat1.cos() * r_lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_M * c
}

/// 3D slant distance taking elevation difference into account.
pub fn slant_distance_3d_m(lat1: f64, lon1: f64, alt1: f64, lat2: f64, lon2: f64, alt2: f64) -> f64 {
    let h_dist = haversine_distance_m(lat1, lon1, lat2, lon2);
    let v_dist = alt2 - alt1;
    (h_dist.powi(2) + v_dist.powi(2)).sqrt()
}

/// Initial true bearing from coordinate 1 to 2 in degrees [0, 360).
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let y = d_lon.sin() * r_lat2.cos();
    let x = r_lat1.cos() * r_lat2.sin() - r_lat1.sin() * r_lat2.cos() * d_lon.cos();
    let brg = y.atan2(x).to_degrees();
    (brg + 360.0) % 360.0
}

/// Cross-track error in meters from point P to geodesic track segment A->B.
/// Positive value indicates right of track; negative indicates left of track.
pub fn cross_track_error_m(p_lat: f64, p_lon: f64, a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let d13 = haversine_distance_m(a_lat, a_lon, p_lat, p_lon) / EARTH_RADIUS_M;
    let brg13 = initial_bearing_deg(a_lat, a_lon, p_lat, p_lon).to_radians();
    let brg12 = initial_bearing_deg(a_lat, a_lon, b_lat, b_lon).to_radians();

    let xt = (d13.sin() * (brg13 - brg12).sin()).asin();
    xt * EARTH_RADIUS_M
}

/// Along-track distance in meters from start waypoint A toward B for point P.
pub fn along_track_distance_m(p_lat: f64, p_lon: f64, a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let d13 = haversine_distance_m(a_lat, a_lon, p_lat, p_lon) / EARTH_RADIUS_M;
    let d_xt = cross_track_error_m(p_lat, p_lon, a_lat, a_lon, b_lat, b_lon) / EARTH_RADIUS_M;
    let at = (d13.cos() / d_xt.cos()).acos();
    if at.is_nan() {
        0.0
    } else {
        at * EARTH_RADIUS_M
    }
}

/// Planned mission waypoint representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteWaypoint {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub planned_speed_mps: Option<f64>,
    pub loiter_sec: f64,
}
