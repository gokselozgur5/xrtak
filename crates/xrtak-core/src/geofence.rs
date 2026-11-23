//! Geofence polygon ray-casting containment checker.

use serde::{Deserialize, Serialize};

/// Point coordinate for geofencing calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeofencePoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}

impl GeofencePoint {
    pub fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: None }
    }

    pub fn new_3d(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z: Some(z) }
    }
}

/// 2D axis-aligned bounding box for fast containment pre-filtering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn from_points(points: &[GeofencePoint]) -> Option<Self> {
        let first = points.first()?;
        let mut min_x = first.x;
        let mut max_x = first.x;
        let mut min_y = first.y;
        let mut max_y = first.y;
        for p in points.iter().skip(1) {
            if p.x < min_x {
                min_x = p.x;
            }
            if p.x > max_x {
                max_x = p.x;
            }
            if p.y < min_y {
                min_y = p.y;
            }
            if p.y > max_y {
                max_y = p.y;
            }
        }
        Some(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }
    pub fn contains_2d(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

/// Polygonal geofence volume with optional altitude limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeofencePolygon {
    pub id: String,
    pub name: String,
    pub vertices: Vec<GeofencePoint>,
    pub floor_m: Option<f64>,
    pub ceiling_m: Option<f64>,
    pub bbox: Option<BoundingBox>,
}

impl GeofencePolygon {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        vertices: Vec<GeofencePoint>,
    ) -> Self {
        let bbox = BoundingBox::from_points(&vertices);
        Self {
            id: id.into(),
            name: name.into(),
            vertices,
            floor_m: None,
            ceiling_m: None,
            bbox,
        }
    }
    pub fn with_altitude_bounds(mut self, floor_m: Option<f64>, ceiling_m: Option<f64>) -> Self {
        self.floor_m = floor_m;
        self.ceiling_m = ceiling_m;
        self
    }
    /// Tests 2D point containment using Jordan curve ray-casting.
    pub fn contains_2d(&self, x: f64, y: f64) -> bool {
        if self.vertices.len() < 3 {
            return false;
        }
        if let Some(ref bbox) = self.bbox {
            if !bbox.contains_2d(x, y) {
                return false;
            }
        }
        let en: Vec<(f64, f64)> = self.vertices.iter().map(|p| (p.x, p.y)).collect();
        crate::zones::point_in_polygon_en(x, y, &en)
    }
    /// Checks if a 3D point is contained within polygon and altitude bounds.
    pub fn contains_point(&self, point: &GeofencePoint) -> bool {
        if !self.contains_2d(point.x, point.y) {
            return false;
        }
        if let Some(z) = point.z {
            if let Some(floor) = self.floor_m {
                if z < floor {
                    return false;
                }
            }
            if let Some(ceiling) = self.ceiling_m {
                if z > ceiling {
                    return false;
                }
            }
        }
        true
    }
    /// Filters tracks that breach the geofence perimeter.
    pub fn find_breaching_tracks<'a>(&self, tracks: &'a [(String, GeofencePoint)]) -> Vec<&'a str> {
        tracks
            .iter()
            .filter(|(_, pt)| self.contains_point(pt))
            .map(|(uid, _)| uid.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degenerate_polygon() {
        let empty = GeofencePolygon::new("poly-0", "Empty", vec![]);
        assert!(!empty.contains_2d(0.0, 0.0));
        let two_pts = vec![
            GeofencePoint::new_2d(0.0, 0.0),
            GeofencePoint::new_2d(1.0, 1.0),
        ];
        let poly2 = GeofencePolygon::new("poly-1", "TwoPts", two_pts);
        assert!(!poly2.contains_2d(0.5, 0.5));
    }
    #[test]
    fn test_convex_polygon_containment() {
        let vertices = vec![
            GeofencePoint::new_2d(0.0, 0.0),
            GeofencePoint::new_2d(10.0, 0.0),
            GeofencePoint::new_2d(10.0, 10.0),
            GeofencePoint::new_2d(0.0, 10.0),
        ];
        let poly = GeofencePolygon::new("box-1", "Box", vertices);
        assert!(poly.contains_2d(5.0, 5.0));
        assert!(!poly.contains_2d(15.0, 5.0));
        assert!(!poly.contains_2d(-1.0, 5.0));
    }
    #[test]
    fn test_concave_polygon_containment() {
        let vertices = vec![
            GeofencePoint::new_2d(0.0, 0.0),
            GeofencePoint::new_2d(10.0, 0.0),
            GeofencePoint::new_2d(10.0, 5.0),
            GeofencePoint::new_2d(5.0, 5.0),
            GeofencePoint::new_2d(5.0, 10.0),
            GeofencePoint::new_2d(0.0, 10.0),
        ];
        let poly = GeofencePolygon::new("l-1", "L-Shape", vertices);
        assert!(poly.contains_2d(2.0, 2.0));
        assert!(poly.contains_2d(2.0, 8.0));
        assert!(!poly.contains_2d(8.0, 8.0));
    }
    #[test]
    fn test_altitude_bounds() {
        let vertices = vec![
            GeofencePoint::new_2d(0.0, 0.0),
            GeofencePoint::new_2d(10.0, 0.0),
            GeofencePoint::new_2d(10.0, 10.0),
            GeofencePoint::new_2d(0.0, 10.0),
        ];
        let poly = GeofencePolygon::new("box-alt", "BoxAlt", vertices)
            .with_altitude_bounds(Some(100.0), Some(500.0));
        assert!(poly.contains_point(&GeofencePoint::new_3d(5.0, 5.0, 250.0)));
        assert!(!poly.contains_point(&GeofencePoint::new_3d(5.0, 5.0, 50.0)));
        assert!(!poly.contains_point(&GeofencePoint::new_3d(5.0, 5.0, 600.0)));
    }
    #[test]
    fn test_find_breaching_tracks() {
        let vertices = vec![
            GeofencePoint::new_2d(0.0, 0.0),
            GeofencePoint::new_2d(10.0, 0.0),
            GeofencePoint::new_2d(10.0, 10.0),
            GeofencePoint::new_2d(0.0, 10.0),
        ];
        let poly = GeofencePolygon::new("box-t", "Tracks", vertices);
        let tracks = vec![
            ("TRK-1".to_string(), GeofencePoint::new_2d(5.0, 5.0)),
            ("TRK-2".to_string(), GeofencePoint::new_2d(15.0, 5.0)),
        ];
        let breaches = poly.find_breaching_tracks(&tracks);
        assert_eq!(breaches, vec!["TRK-1"]);
    }
}
