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

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn overlaps(&self, other: &GeoRasterExtent) -> bool {
        !(self.max_x < other.min_x
            || self.min_x > other.max_x
            || self.max_y < other.min_y
            || self.min_y > other.max_y)
    }
}

/// Parses a 6-parameter ESRI World File text string.
pub fn parse_world_file(content: &str) -> Result<WorldFileTransform, String> {
    let tokens: Vec<&str> = content
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.len() != 6 {
        return Err(format!("Expected 6 numerical values, found {}", tokens.len()));
    }

    let parse_val = |idx: usize, name: &'static str| -> Result<f64, String> {
        tokens[idx].parse::<f64>().map_err(|e| format!("{}: {}", name, e))
    };

    let a = parse_val(0, "Line 1")?;
    let d = parse_val(1, "Line 2")?;
    let b = parse_val(2, "Line 3")?;
    let e = parse_val(3, "Line 4")?;
    let c = parse_val(4, "Line 5")?;
    let f = parse_val(5, "Line 6")?;

    Ok(WorldFileTransform::new(a, d, b, e, c, f))
}

/// Calculates spatial 4-corner footprint and bounding box from affine transform.
pub fn compute_raster_extent(
    transform: &WorldFileTransform,
    width_px: usize,
    height_px: usize,
) -> Result<GeoRasterExtent, String> {
    if width_px == 0 || height_px == 0 {
        return Err("Raster dimensions must be greater than zero".to_string());
    }

    let min_p = -0.5;
    let max_px = width_px as f64 - 0.5;
    let max_py = height_px as f64 - 0.5;

    let top_left = transform.pixel_to_world(min_p, min_p);
    let top_right = transform.pixel_to_world(max_px, min_p);
    let bottom_left = transform.pixel_to_world(min_p, max_py);
    let bottom_right = transform.pixel_to_world(max_px, max_py);
