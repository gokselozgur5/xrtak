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
        (
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
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
        return Err(format!(
            "Expected 6 numerical values, found {}",
            tokens.len()
        ));
    }

    let parse_val = |idx: usize, name: &'static str| -> Result<f64, String> {
        tokens[idx]
            .parse::<f64>()
            .map_err(|e| format!("{}: {}", name, e))
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

    let xs = [top_left.0, top_right.0, bottom_left.0, bottom_right.0];
    let ys = [top_left.1, top_right.1, bottom_left.1, bottom_right.1];

    let mut min_x = xs[0];
    let mut max_x = xs[0];
    let mut min_y = ys[0];
    let mut max_y = ys[0];

    for &x in &xs[1..] {
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
    }
    for &y in &ys[1..] {
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }

    Ok(GeoRasterExtent {
        min_x,
        max_x,
        min_y,
        max_y,
        width_px,
        height_px,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    })
}

/// Parses GeoTIFF ModelTransformationTag (Tag 34264) 4x4 matrix into affine transform.
pub fn parse_geotiff_model_transform_4x4(matrix: &[f64; 16]) -> Result<WorldFileTransform, String> {
    let a = matrix[0];
    let b = matrix[1];
    let c = matrix[3];
    let d = matrix[4];
    let e = matrix[5];
    let f = matrix[7];

    let det = a * e - b * d;
    if det.abs() < 1e-14 {
        return Err("Degenerate GeoTIFF model transformation matrix".to_string());
    }

    Ok(WorldFileTransform::new(a, d, b, e, c, f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_file_parsing_standard() {
        let content = "0.5\n0.0\n0.0\n-0.5\n32.85\n39.92\n";
        let tf = parse_world_file(content).expect("valid tfw");
        assert_eq!(tf.a, 0.5);
        assert_eq!(tf.d, 0.0);
        assert_eq!(tf.b, 0.0);
        assert_eq!(tf.e, -0.5);
        assert_eq!(tf.c, 32.85);
        assert_eq!(tf.f, 39.92);
        assert!(tf.is_north_up());
    }

    #[test]
    fn test_world_file_parsing_invalid() {
        let err1 = parse_world_file("0.5 0.0 0.0").unwrap_err();
        assert_eq!(err1, "Expected 6 numerical values, found 3");

        let err2 = parse_world_file("0.5 0.0 0.0 -0.5 32.85 bad_float").unwrap_err();
        assert!(err2.starts_with("Line 6:"));
    }

    #[test]
    fn test_pixel_to_world_roundtrip() {
        let tf = WorldFileTransform::new(2.5, 0.1, -0.1, -2.5, 500000.0, 4200000.0);
        let px = 120.0;
        let py = 85.0;

        let (wx, wy) = tf.pixel_to_world(px, py);
        let (r_px, r_py) = tf.world_to_pixel(wx, wy).expect("invertible");

        assert!((r_px - px).abs() < 1e-6);
        assert!((r_py - py).abs() < 1e-6);
    }

    #[test]
    fn test_singular_transform_error() {
        let singular = WorldFileTransform::new(0.0, 0.0, 0.0, 0.0, 10.0, 10.0);
        let err = singular.world_to_pixel(10.0, 10.0).unwrap_err();
        assert_eq!(err, "Singular non-invertible affine transform matrix");
    }

    #[test]
    fn test_compute_raster_extent() {
        let tf = WorldFileTransform::from_scale_and_origin(1.0, 1.0, 100.0, 200.0);
        let ext = compute_raster_extent(&tf, 100, 50).expect("valid extent");

        assert_eq!(ext.width_px, 100);
        assert_eq!(ext.height_px, 50);
        assert!(ext.span_x() > 99.0);
        assert!(ext.span_y() > 49.0);
        assert!(ext.contains(150.0, 175.0));
        assert!(!ext.contains(50.0, 250.0));
    }

    #[test]
    fn test_extent_overlap() {
        let tf1 = WorldFileTransform::from_scale_and_origin(1.0, 1.0, 0.0, 10.0);
        let ext1 = compute_raster_extent(&tf1, 10, 10).expect("valid ext1");

        let tf2 = WorldFileTransform::from_scale_and_origin(1.0, 1.0, 5.0, 5.0);
        let ext2 = compute_raster_extent(&tf2, 10, 10).expect("valid ext2");

        let tf3 = WorldFileTransform::from_scale_and_origin(1.0, 1.0, 50.0, 50.0);
        let ext3 = compute_raster_extent(&tf3, 10, 10).expect("valid ext3");

        assert!(ext1.overlaps(&ext2));
        assert!(ext2.overlaps(&ext1));
        assert!(!ext1.overlaps(&ext3));
    }

    #[test]
    fn test_geotiff_model_transform_4x4() {
        let matrix = [
            1.5, 0.0, 0.0, 32.0, 0.0, -1.5, 0.0, 40.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let tf = parse_geotiff_model_transform_4x4(&matrix).expect("valid 4x4 matrix");
        assert_eq!(tf.a, 1.5);
        assert_eq!(tf.e, -1.5);
        assert_eq!(tf.c, 32.0);
        assert_eq!(tf.f, 40.0);
        assert!(tf.is_north_up());
    }

    #[test]
    fn test_to_world_file_content_serialization() {
        let tf = WorldFileTransform::new(0.25, 0.0, 0.0, -0.25, 50.0, 60.0);
        let str_rep = tf.to_world_file_content();
        let parsed = parse_world_file(&str_rep).expect("roundtrip parse");
        assert_eq!(tf, parsed);
    }
}
