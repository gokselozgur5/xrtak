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
