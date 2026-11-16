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

    pub fn with_callsign(mut self, callsign: impl Into<String>) -> Self {
        self.callsign = Some(callsign.into());
        self
    }

    pub fn with_cot_type(mut self, cot_type: impl Into<String>) -> Self {
        self.cot_type = Some(cot_type.into());
        self
    }

    pub fn with_elevation(mut self, elevation_m: f64) -> Self {
        self.elevation = Some(elevation_m);
        self
    }

    pub fn with_time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }

    pub fn with_remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }

    /// Validate coordinate ranges (-180..=180 longitude, -90..=90 latitude).
    pub fn validate(&self) -> Result<(), GeoPackageError> {
        if !(-180.0..=180.0).contains(&self.lon) || !(-90.0..=90.0).contains(&self.lat) {
            return Err(GeoPackageError::InvalidCoordinates {
                lon: self.lon,
                lat: self.lat,
            });
        }
        Ok(())
    }
}

/// 2D/3D Geographic Bounding Envelope for GeoPackage contents and headers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoPackageBounds {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
    pub min_elev: Option<f64>,
    pub max_elev: Option<f64>,
}

impl GeoPackageBounds {
    pub fn from_point(lon: f64, lat: f64, elev: Option<f64>) -> Self {
        Self {
            min_lon: lon,
            min_lat: lat,
            max_lon: lon,
            max_lat: lat,
            min_elev: elev,
            max_elev: elev,
        }
    }
