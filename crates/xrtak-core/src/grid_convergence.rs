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

