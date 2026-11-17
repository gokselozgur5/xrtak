//! Tactical route navigation waypoint sequence generator and ETA estimator.

use serde::{Deserialize, Serialize};

/// Earth radius in meters for geodetic calculations.
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;
