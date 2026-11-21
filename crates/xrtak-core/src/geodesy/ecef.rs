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

/// Converts ECEF Cartesian coordinates to WGS84 geodetic coordinates (Bowring's algorithm).
pub fn ecef_to_geodetic(ecef: &EcefCoord) -> GeodeticCoord {
    let p = (ecef.x_m * ecef.x_m + ecef.y_m * ecef.y_m).sqrt();
    if p < 1e-9 {
        let lat = if ecef.z_m >= 0.0 { 90.0 } else { -90.0 };
        return GeodeticCoord::new(lat, 0.0, ecef.z_m.abs() - WGS84_B);
    }

    let theta = (ecef.z_m * WGS84_A).atan2(p * WGS84_B);
    let s_th = theta.sin();
    let c_th = theta.cos();

    let num = ecef.z_m + WGS84_E_PRIME2 * WGS84_B * s_th * s_th * s_th;
    let den = p - WGS84_E2 * WGS84_A * c_th * c_th * c_th;
    let lat_rad = num.atan2(den);
    let lon_rad = ecef.y_m.atan2(ecef.x_m);

    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let alt_m =
        p * cos_lat + ecef.z_m * sin_lat - WGS84_A * (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();

    GeodeticCoord::new(lat_rad.to_degrees(), lon_rad.to_degrees(), alt_m)
}
