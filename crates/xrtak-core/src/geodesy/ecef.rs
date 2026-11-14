use serde::{Deserialize, Serialize};

pub const WGS84_A: f64 = 6_378_137.0;
pub const WGS84_F: f64 = 1.0 / 298.257223563;
pub const WGS84_B: f64 = WGS84_A * (1.0 - WGS84_F);
pub const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F;
pub const WGS84_E_PRIME2: f64 = (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B);

/// WGS84 geodetic coordinates (latitude, longitude in degrees, altitude in meters).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeodeticCoord {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: f64,
}

impl GeodeticCoord {
    pub const fn new(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Self {
        Self {
            lat_deg,
            lon_deg,
            alt_m,
        }
    }
}

/// Earth-Centered, Earth-Fixed (ECEF) Cartesian coordinates in meters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EcefCoord {
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
}

impl EcefCoord {
    pub const fn new(x_m: f64, y_m: f64, z_m: f64) -> Self {
        Self { x_m, y_m, z_m }
    }
}
