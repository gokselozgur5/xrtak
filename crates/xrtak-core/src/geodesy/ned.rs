use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NedCoord {
    pub north_m: f64,
    pub east_m: f64,
    pub down_m: f64,
}

impl NedCoord {
    pub const fn new(north_m: f64, east_m: f64, down_m: f64) -> Self {
        Self {
            north_m,
            east_m,
            down_m,
        }
    }
}

use crate::geodesy::ecef::{geodetic_to_ecef, EcefCoord, GeodeticCoord};

/// Reference origin for local North-East-Down (NED) coordinate frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NedOrigin {
    pub geodetic: GeodeticCoord,
    pub ecef: EcefCoord,
}

impl NedOrigin {
    pub fn new(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Self {
        let geodetic = GeodeticCoord::new(lat_deg, lon_deg, alt_m);
        let ecef = geodetic_to_ecef(&geodetic);
        Self { geodetic, ecef }
    }
}

/// Transforms an ECEF point to NED coordinates relative to the given origin.
pub fn ecef_to_ned(point: &EcefCoord, origin: &NedOrigin) -> NedCoord {
    let dx = point.x_m - origin.ecef.x_m;
    let dy = point.y_m - origin.ecef.y_m;
    let dz = point.z_m - origin.ecef.z_m;

    let lat_rad = origin.geodetic.lat_deg.to_radians();
    let lon_rad = origin.geodetic.lon_deg.to_radians();
    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let sin_lon = lon_rad.sin();
    let cos_lon = lon_rad.cos();

    let north = -sin_lat * cos_lon * dx - sin_lat * sin_lon * dy + cos_lat * dz;
    let east = -sin_lon * dx + cos_lon * dy;
    let down = -cos_lat * cos_lon * dx - cos_lat * sin_lon * dy - sin_lat * dz;

    NedCoord::new(north, east, down)
}
