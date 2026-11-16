//! GeoTIFF world file (.tfw / .wld) and spatial extent bounds parser.

use serde::{Deserialize, Serialize};

/// 6-parameter affine georeferencing transform matrix (ESRI World File specification).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldFileTransform {
    pub a: f64,
    pub d: f64,
    pub b: f64,
    pub e: f64,
    pub c: f64,
    pub f: f64,
}
