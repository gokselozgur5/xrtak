use crate::GeoOrigin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorPolygonResult {
    pub name: String,
    pub input_waypoint_count: usize,
    pub simplified_waypoint_count: usize,
    pub swath_width_m: f64,
    pub area_km2: f64,
    pub simplified_en: Vec<(f64, f64)>,
    pub polygon_ring_en: Vec<(f64, f64)>,
    pub polygon_ring_lla: Vec<(f64, f64)>,
    pub kml_polygon: String,
}

/// Computes ground sensor swath width (meters) from AGL altitude and either horizontal FOV (degrees)
/// or pinhole camera focal length (`focal_px`, `sensor_width_px`) (`REQ-COR-01`).
pub fn compute_camera_swath_width_m(
    agl_m: f64,
    hfov_deg: Option<f64>,
    focal_px: Option<f64>,
    sensor_width_px: Option<f64>,
) -> f64 {
    let alt = agl_m.max(10.0);
    if let (Some(f), Some(w)) = (focal_px, sensor_width_px) {
        if f > 1.0 && w > 1.0 {
            return (alt * (w / f)).clamp(20.0, 5000.0);
        }
    }
    let fov_rad = hfov_deg.unwrap_or(54.0).clamp(5.0, 150.0).to_radians();
    (2.0 * alt * (fov_rad * 0.5).tan()).clamp(20.0, 5000.0)
}

fn perpendicular_distance_m(pt: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-9 {
        return ((pt.0 - a.0).powi(2) + (pt.1 - a.1).powi(2)).sqrt();
    }
    let t = (((pt.0 - a.0) * dx + (pt.1 - a.1) * dy) / len_sq).clamp(0.0, 1.0);
    let proj_x = a.0 + t * dx;
    let proj_y = a.1 + t * dy;
    ((pt.0 - proj_x).powi(2) + (pt.1 - proj_y).powi(2)).sqrt()
}

/// Simplifies an ENU polyline using the Ramer-Douglas-Peucker (RDP) algorithm (`REQ-COR-01`).
pub fn simplify_polyline_rdp(points: &[(f64, f64)], epsilon_m: f64) -> Vec<(f64, f64)> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut max_dist = 0.0;
    let mut split_idx = 0usize;
    let first = points[0];
    let last = points[points.len() - 1];

    for (i, &pt) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let d = perpendicular_distance_m(pt, first, last);
        if d > max_dist {
            max_dist = d;
            split_idx = i;
        }
    }

    if max_dist > epsilon_m.max(0.5) {
        let mut left = simplify_polyline_rdp(&points[..=split_idx], epsilon_m);
        let right = simplify_polyline_rdp(&points[split_idx..], epsilon_m);
        left.pop();
        left.extend_from_slice(&right);
        left
    } else {
        vec![first, last]
    }
}

/// Builds a closed 2D buffered corridor polygon ring `(east_m, north_m)` around a polyline (`REQ-COR-02`).
pub fn build_buffered_corridor_polygon(
    centerline_en: &[(f64, f64)],
    half_width_m: f64,
) -> Vec<(f64, f64)> {
    let hw = half_width_m.max(5.0);
    if centerline_en.is_empty() {
        return Vec::new();
    }
    if centerline_en.len() == 1 {
        let (x, y) = centerline_en[0];
        return vec![
            (x - hw, y - hw),
            (x + hw, y - hw),
            (x + hw, y + hw),
            (x - hw, y + hw),
            (x - hw, y - hw),
        ];
    }

    let n = centerline_en.len();
    let mut seg_normals = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let dx = centerline_en[i + 1].0 - centerline_en[i].0;
        let dy = centerline_en[i + 1].1 - centerline_en[i].1;
        let len = (dx * dx + dy * dy).sqrt().max(1e-6);
        // Left normal (-dy, dx)
        seg_normals.push((-dy / len, dx / len));
    }

    let mut left_side = Vec::with_capacity(n);
    let mut right_side = Vec::with_capacity(n);

    for i in 0..n {
        let (nx, ny) = if i == 0 {
            seg_normals[0]
        } else if i == n - 1 {
            seg_normals[n - 2]
        } else {
            let (n0x, n0y) = seg_normals[i - 1];
            let (n1x, n1y) = seg_normals[i];
            let mx = n0x + n1x;
            let my = n0y + n1y;
            let mlen = (mx * mx + my * my).sqrt();
            if mlen < 1e-3 {
                (n1x, n1y)
            } else {
                // Clamp miter join factor to 1.8x half-width to prevent sharp spikes
                let dot = (mx / mlen) * n1x + (my / mlen) * n1y;
                let scale = (1.0 / dot.max(0.55)).min(1.8);
                ((mx / mlen) * scale, (my / mlen) * scale)
            }
        };
        let (cx, cy) = centerline_en[i];
        left_side.push((cx + nx * hw, cy + ny * hw));
        right_side.push((cx - nx * hw, cy - ny * hw));
    }

    right_side.reverse();
    let mut ring = left_side;
    ring.extend(right_side);
    if let Some(&first) = ring.first() {
        ring.push(first);
    }
    ring
}

fn polygon_area_m2(ring_en: &[(f64, f64)]) -> f64 {
    if ring_en.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..(ring_en.len() - 1) {
        sum += ring_en[i].0 * ring_en[i + 1].1 - ring_en[i + 1].0 * ring_en[i].1;
    }
    (sum * 0.5).abs()
}

/// Evaluates RDP simplification, optical swath width, buffered corridor ring, and KML `<Polygon>` (`ICD-17`).
// Argument bundling is tracked as XR-074.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_swath_corridor(
    name: impl Into<String>,
    origin: &GeoOrigin,
    waypoints_en: &[(f64, f64)],
    rdp_epsilon_m: f64,
    agl_m: f64,
    hfov_deg: Option<f64>,
    focal_px: Option<f64>,
    sensor_width_px: Option<f64>,
) -> CorridorPolygonResult {
    let name_str = name.into();
    let simplified = simplify_polyline_rdp(waypoints_en, rdp_epsilon_m);
    let swath_width_m = compute_camera_swath_width_m(agl_m, hfov_deg, focal_px, sensor_width_px);
    let ring_en = build_buffered_corridor_polygon(&simplified, swath_width_m * 0.5);
    let area_km2 = polygon_area_m2(&ring_en) / 1_000_000.0;

    let mut ring_lla = Vec::with_capacity(ring_en.len());
    let mut coords_str = Vec::with_capacity(ring_en.len());
    for &(e, n) in &ring_en {
        let (lat, lon, _) = origin.enu_to_lla(e, 0.0, n);
        ring_lla.push(((lat * 1e6).round() / 1e6, (lon * 1e6).round() / 1e6));
        coords_str.push(format!("{lon:.6},{lat:.6},0"));
    }

    let kml_polygon = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <name>{name_str}</name>
    <Placemark>
      <name>{name_str} Corridor ({swath:.0}m swath)</name>
      <Polygon>
        <outerBoundaryIs>
          <LinearRing>
            <coordinates>{coords}</coordinates>
          </LinearRing>
        </outerBoundaryIs>
      </Polygon>
    </Placemark>
  </Document>
</kml>"#,
        swath = swath_width_m,
        coords = coords_str.join(" ")
    );

    CorridorPolygonResult {
        name: name_str,
        input_waypoint_count: waypoints_en.len(),
        simplified_waypoint_count: simplified.len(),
        swath_width_m: (swath_width_m * 10.0).round() / 10.0,
        area_km2: (area_km2 * 1000.0).round() / 1000.0,
        simplified_en: simplified,
        polygon_ring_en: ring_en,
        polygon_ring_lla: ring_lla,
        kml_polygon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_swath_width_and_rdp_simplification() {
        // 90 deg HFOV at 150m AGL -> 2 * 150 * tan(45) = 300m swath
        let w = compute_camera_swath_width_m(150.0, Some(90.0), None, None);
        assert!((w - 300.0).abs() < 1e-4);

        // Collinear noisy points along X axis from 0..1000m, plus a 90 deg turn to (1000, 500)
        let raw = vec![
            (0.0, 0.0),
            (250.0, 1.2),
            (500.0, -0.8),
            (750.0, 0.5),
            (1000.0, 0.0),
            (1000.0, 500.0),
        ];
        let simp = simplify_polyline_rdp(&raw, 5.0);
        assert_eq!(simp.len(), 3);
        assert_eq!(simp[0], (0.0, 0.0));
        assert_eq!(simp[1], (1000.0, 0.0));
        assert_eq!(simp[2], (1000.0, 500.0));
    }

    #[test]
    fn test_buffered_corridor_polygon_and_kml() {
        let origin = GeoOrigin::default();
        let line = vec![(0.0, 0.0), (1000.0, 0.0)];
        let res = evaluate_swath_corridor(
            "TEST-CORRIDOR",
            &origin,
            &line,
            5.0,
            100.0,
            Some(90.0),
            None,
            None,
        );
        // 100m AGL @ 90 deg HFOV -> 200m swath width -> 1000m * 200m = 0.200 km^2
        assert!((res.swath_width_m - 200.0).abs() < 0.2);
        assert!((res.area_km2 - 0.200).abs() < 0.01);
        assert!(res.kml_polygon.contains("<Polygon>"));
        assert!(res.polygon_ring_en.len() >= 5);
    }
}
