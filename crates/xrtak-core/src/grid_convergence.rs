//! Grid convergence angle and scale factor calculator for UTM and UPS.
//!
//! Provides geodetic calculations on the WGS-84 ellipsoid for:
//! - Universal Transverse Mercator (UTM) zones (80S to 84N)
//! - Universal Polar Stereographic (UPS) North (>= 80N) and South (<= -80S)
//! - Grid convergence (angle between True North and Grid North)
//! - Point scale factor (ratio of grid distance to ground distance)
//! - True and Grid azimuth / distance conversions

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// WGS-84 semi-major axis in meters.
pub const WGS84_A: f64 = 6_378_137.0;

/// WGS-84 reciprocal flattening.
pub const WGS84_RECIPROCAL_F: f64 = 298.257_223_563;

/// WGS-84 first eccentricity squared: e^2 = 2f - f^2.
pub const WGS84_E2: f64 = 0.006_694_379_990_141_316_5;

/// WGS-84 second eccentricity squared: e'^2 = e^2 / (1 - e^2).
pub const WGS84_EP2: f64 = 0.006_739_496_742_276_434;

/// UTM central meridian scale factor.
pub const UTM_K0: f64 = 0.9996;

/// UPS pole scale factor.
pub const UPS_K0: f64 = 0.9940;

/// Grid projection system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridSystem {
    /// Universal Transverse Mercator with zone (1..=60) and hemisphere.
    Utm { zone: u8, north: bool },
    /// Universal Polar Stereographic (North polar cap, >= 80N).
    UpsNorth,
    /// Universal Polar Stereographic (South polar cap, <= -80S).
    UpsSouth,
}

/// Errors during grid convergence and scale factor computation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridError {
    /// Latitude exceeds [-90.0, 90.0] degrees.
    InvalidLatitude(String),
    /// Longitude exceeds [-180.0, 180.0] degrees.
    InvalidLongitude(String),
    /// UTM zone is outside 1..=60.
    InvalidUtmZone(u8),
    /// Latitude is outside the valid UTM band (-80.0S to 84.0N).
    LatitudeOutOfUtmRange(String),
    /// Latitude is outside the valid UPS band.
    LatitudeOutOfUpsRange(String),
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLatitude(msg) => write!(f, "invalid latitude: {}", msg),
            Self::InvalidLongitude(msg) => write!(f, "invalid longitude: {}", msg),
            Self::InvalidUtmZone(z) => write!(f, "invalid UTM zone: {} (must be 1..=60)", z),
            Self::LatitudeOutOfUtmRange(msg) => write!(f, "latitude out of UTM range: {}", msg),
            Self::LatitudeOutOfUpsRange(msg) => write!(f, "latitude out of UPS range: {}", msg),
        }
    }
}

impl std::error::Error for GridError {}

/// Result of grid convergence and scale factor calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridSolution {
    /// Grid projection system (UTM zone or UPS polar aspect).
    pub system: GridSystem,
    /// Grid convergence angle in decimal degrees (Grid North - True North).
    /// Positive when Grid North is clockwise (East) of True North.
    pub convergence_deg: f64,
    /// Grid convergence angle in radians.
    pub convergence_rad: f64,
    /// Point scale factor k (grid length / ellipsoidal surface length).
    pub scale_factor: f64,
    /// Central meridian longitude in decimal degrees.
    pub central_meridian_deg: f64,
}

/// Determine the standard UTM zone number (1..=60) for a given longitude in degrees.
pub fn utm_zone_from_lon(lon_deg: f64) -> u8 {
    let mut normalized_lon = lon_deg;
    while normalized_lon < -180.0 {
