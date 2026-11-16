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
            if p.x < min_x { min_x = p.x; }
            if p.x > max_x { max_x = p.x; }
            if p.y < min_y { min_y = p.y; }
            if p.y > max_y { max_y = p.y; }
        }
        Some(Self { min_x, min_y, max_x, max_y })
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
    pub fn new(id: impl Into<String>, name: impl Into<String>, vertices: Vec<GeofencePoint>) -> Self {
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
