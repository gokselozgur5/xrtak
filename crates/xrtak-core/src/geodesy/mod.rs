//! Geodesy module root.

pub mod ecef;
pub mod ned;

pub use ecef::{ecef_to_geodetic, geodetic_to_ecef, EcefCoord, GeodeticCoord};
pub use ned::{ecef_to_ned, ned_to_ecef, NedCoord, NedOrigin};

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
