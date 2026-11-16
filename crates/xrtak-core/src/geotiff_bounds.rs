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

    pub fn world_to_pixel(&self, wx: f64, wy: f64) -> Result<(f64, f64), String> {
        let det = self.a * self.e - self.b * self.d;
        if det.abs() < 1e-14 {
            return Err("Singular non-invertible affine transform matrix".to_string());
        }
        let dx = wx - self.c;
        let dy = wy - self.f;
        let px = (self.e * dx - self.b * dy) / det;
        let py = (-self.d * dx + self.a * dy) / det;
        Ok((px, py))
    }

    pub fn to_world_file_content(&self) -> String {
        format!(
            "{:.10}\n{:.10}\n{:.10}\n{:.10}\n{:.10}\n{:.10}\n",
            self.a, self.d, self.b, self.e, self.c, self.f
        )
    }
}

/// Geographic bounding box and 4-corner footprint of a raster image.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoRasterExtent {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub width_px: usize,
    pub height_px: usize,
    pub top_left: (f64, f64),
    pub top_right: (f64, f64),
    pub bottom_left: (f64, f64),
    pub bottom_right: (f64, f64),
}

impl GeoRasterExtent {
    pub fn center(&self) -> (f64, f64) {
        ((self.min_x + self.max_x) * 0.5, (self.min_y + self.max_y) * 0.5)
    }

    pub fn span_x(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn span_y(&self) -> f64 {
        self.max_y - self.min_y
    }
