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
pub const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F;

/// WGS-84 second eccentricity squared (e'^2 = e^2 / (1 - e^2)).
pub const WGS84_EP2: f64 = WGS84_E2 / (1.0 - WGS84_E2);

/// UTM central scale factor.
pub const UTM_K0: f64 = 0.9996;

/// UTM false easting in meters.
pub const FALSE_EASTING: f64 = 500_000.0;

/// UTM false northing for southern hemisphere in meters.
pub const FALSE_NORTHING_SOUTH: f64 = 10_000_000.0;

/// MGRS latitude bands covering -80.0 to 84.0 degrees latitude.
pub const LATITUDE_BANDS: [(char, f64, f64); 20] = [
    ('C', -80.0, -72.0),
    ('D', -72.0, -64.0),
    ('E', -64.0, -56.0),
    ('F', -56.0, -48.0),
    ('G', -48.0, -40.0),
    ('H', -40.0, -32.0),
    ('J', -32.0, -24.0),
