//! Line-of-sight Fresnel zone clearance and RF link budget calculator.
//!
//! Evaluates Fresnel zone radii, Earth curvature bulge, obstacle clearances,
//! knife-edge diffraction loss, and total RF link budget margins.

use serde::{Deserialize, Serialize};

/// Speed of light in vacuum (meters per second).
pub const SPEED_OF_LIGHT_MPS: f64 = 299_792_458.0;

/// Mean Earth radius in meters (WGS-84 reference).
pub const DEFAULT_EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Standard atmospheric refraction k-factor (4/3 Earth model).
pub const DEFAULT_K_FACTOR: f64 = 4.0 / 3.0;
