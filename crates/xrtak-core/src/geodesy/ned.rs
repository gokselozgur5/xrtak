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
