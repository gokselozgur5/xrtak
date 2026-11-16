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

impl WorldFileTransform {
    pub const fn new(a: f64, d: f64, b: f64, e: f64, c: f64, f: f64) -> Self {
        Self { a, d, b, e, c, f }
    }

    pub fn from_scale_and_origin(scale_x: f64, scale_y: f64, origin_x: f64, origin_y: f64) -> Self {
        Self {
            a: scale_x.abs(),
            d: 0.0,
            b: 0.0,
            e: -scale_y.abs(),
            c: origin_x,
            f: origin_y,
        }
    }

    pub fn is_north_up(&self) -> bool {
        self.b.abs() < 1e-12 && self.d.abs() < 1e-12 && self.e < 0.0
    }

    pub fn pixel_to_world(&self, px: f64, py: f64) -> (f64, f64) {
        let wx = self.a * px + self.b * py + self.c;
        let wy = self.d * px + self.e * py + self.f;
        (wx, wy)
    }
