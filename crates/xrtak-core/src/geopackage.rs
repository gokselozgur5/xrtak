//! OGC GeoPackage (GPKG) waypoint feature writer and binary geometry encoder (ICD-GPKG-WP-01).

use serde::{Deserialize, Serialize};

/// Error types encountered during GeoPackage geometry encoding or SQL generation.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoPackageError {
    InvalidMagic,
    UnsupportedVersion(u8),
    InvalidGeometryBlob(String),
    InvalidCoordinates { lon: f64, lat: f64 },
    EmptyDataset,
}

impl std::fmt::Display for GeoPackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "invalid geopackage geometry magic bytes"),
            Self::UnsupportedVersion(v) => {
                write!(f, "unsupported geopackage binary version: {v}")
            }
            Self::InvalidGeometryBlob(msg) => write!(f, "invalid geometry blob: {msg}"),
            Self::InvalidCoordinates { lon, lat } => {
                write!(f, "coordinates out of bounds: lon={lon}, lat={lat}")
            }
            Self::EmptyDataset => write!(f, "dataset contains no waypoint records"),
        }
    }
}

impl std::error::Error for GeoPackageError {}

/// Tactical waypoint feature record for GeoPackage export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoPackageWaypoint {
    pub id: Option<i64>,
    pub name: String,
    pub callsign: Option<String>,
    pub cot_type: Option<String>,
    pub lon: f64,
    pub lat: f64,
    pub elevation: Option<f64>,
    pub time: Option<String>,
    pub remarks: Option<String>,
}

impl GeoPackageWaypoint {
    /// Construct a new waypoint at geographic coordinates (lon, lat in degrees).
    pub fn new(name: impl Into<String>, lon: f64, lat: f64) -> Self {
        Self {
            id: None,
            name: name.into(),
            callsign: None,
            cot_type: None,
            lon,
            lat,
            elevation: None,
            time: None,
            remarks: None,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }
