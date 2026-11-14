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

/// Converts WGS84 geodetic coordinates to ECEF Cartesian coordinates.
pub fn geodetic_to_ecef(geo: &GeodeticCoord) -> EcefCoord {
    let lat_rad = geo.lat_deg.to_radians();
    let lon_rad = geo.lon_deg.to_radians();
    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let sin_lon = lon_rad.sin();
    let cos_lon = lon_rad.cos();

    let n = WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();
    let x = (n + geo.alt_m) * cos_lat * cos_lon;
    let y = (n + geo.alt_m) * cos_lat * sin_lon;
    let z = (n * (1.0 - WGS84_E2) + geo.alt_m) * sin_lat;

    EcefCoord::new(x, y, z)
}
