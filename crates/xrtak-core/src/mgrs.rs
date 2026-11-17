//! Military Grid Reference System (MGRS) precision coordinate converter.
//!
//! Provides bidirectional conversion between:
//! - Geodetic coordinates (WGS-84 latitude / longitude / elevation)
//! - Universal Transverse Mercator (UTM) coordinates
//! - Military Grid Reference System (MGRS) alphanumeric grid references
//!
//! Supports variable precision levels (1m, 10m, 100m, 1km, 10km, 100km),
//! bounding box calculation, center coordinate resolution, and formatting.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// WGS-84 semi-major axis in meters.
pub const WGS84_A: f64 = 6_378_137.0;

/// WGS-84 reciprocal flattening (1 / f).
pub const WGS84_RECIPROCAL_F: f64 = 298.257_223_563;

/// WGS-84 flattening f.
pub const WGS84_F: f64 = 1.0 / WGS84_RECIPROCAL_F;

/// WGS-84 first eccentricity squared (e^2 = 2f - f^2).
