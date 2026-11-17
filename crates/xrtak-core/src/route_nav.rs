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
