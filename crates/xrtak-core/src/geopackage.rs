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

    pub fn update(&mut self, lon: f64, lat: f64, elev: Option<f64>) {
        if lon < self.min_lon {
            self.min_lon = lon;
        }
        if lon > self.max_lon {
            self.max_lon = lon;
        }
        if lat < self.min_lat {
            self.min_lat = lat;
        }
        if lat > self.max_lat {
            self.max_lat = lat;
        }
        if let Some(z) = elev {
            self.min_elev = Some(self.min_elev.map_or(z, |cur| cur.min(z)));
            self.max_elev = Some(self.max_elev.map_or(z, |cur| cur.max(z)));
        }
    }
}

const GPKG_FLAG_LITTLE_ENDIAN: u8 = 0x01;
const GPKG_FLAG_ENVELOPE_NONE: u8 = 0x00;
const WGS84_SRS_ID: i32 = 4326;
const WKB_POINT_2D: u32 = 1;
const WKB_POINT_Z: u32 = 1001;

/// Encode point geometry to standard OGC GeoPackage binary geometry BLOB.
pub fn encode_point_geom(
    lon: f64,
    lat: f64,
    elev: Option<f64>,
    srs_id: i32,
) -> Result<Vec<u8>, GeoPackageError> {
    if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
        return Err(GeoPackageError::InvalidCoordinates { lon, lat });
    }

    let is_3d = elev.is_some();
    let cap = if is_3d { 8 + 1 + 4 + 24 } else { 8 + 1 + 4 + 16 };
    let mut blob = Vec::with_capacity(cap);

    blob.push(0x47);
    blob.push(0x50);
    blob.push(0x00);
    blob.push(GPKG_FLAG_LITTLE_ENDIAN | GPKG_FLAG_ENVELOPE_NONE);
    blob.extend_from_slice(&srs_id.to_le_bytes());

    blob.push(0x01);
    if let Some(z) = elev {
        blob.extend_from_slice(&WKB_POINT_Z.to_le_bytes());
        blob.extend_from_slice(&lon.to_le_bytes());
        blob.extend_from_slice(&lat.to_le_bytes());
        blob.extend_from_slice(&z.to_le_bytes());
    } else {
        blob.extend_from_slice(&WKB_POINT_2D.to_le_bytes());
        blob.extend_from_slice(&lon.to_le_bytes());
        blob.extend_from_slice(&lat.to_le_bytes());
    }

    Ok(blob)
}

/// Decode standard OGC GeoPackage binary geometry BLOB into (lon, lat, Option<elev>).
pub fn decode_point_geom(blob: &[u8]) -> Result<(f64, f64, Option<f64>), GeoPackageError> {
    if blob.len() < 8 {
        return Err(GeoPackageError::InvalidGeometryBlob(
            "header truncated".into(),
        ));
    }
    if blob[0] != 0x47 || blob[1] != 0x50 {
        return Err(GeoPackageError::InvalidMagic);
    }
    if blob[2] != 0x00 {
        return Err(GeoPackageError::UnsupportedVersion(blob[2]));
    }

    let flags = blob[3];
    let envelope_code = (flags >> 1) & 0x07;
    let envelope_len = match envelope_code {
        0 => 0,
        1 => 32,
        2 | 3 => 48,
        4 => 64,
        _ => return Err(GeoPackageError::InvalidGeometryBlob("invalid envelope indicator".into())),
    };

    let wkb_offset = 8 + envelope_len;
    if blob.len() < wkb_offset + 5 {
        return Err(GeoPackageError::InvalidGeometryBlob(
            "wkb payload truncated".into(),
        ));
    }

    let wkb_bytes = &blob[wkb_offset..];
    let wkb_le = wkb_bytes[0] == 1;

    let read_u32 = |slice: &[u8]| -> u32 {
        if wkb_le {
            u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
        } else {
            u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };

    let geom_type = read_u32(&wkb_bytes[1..5]);
    let read_f64 = |slice: &[u8]| -> f64 {
        if wkb_le {
            f64::from_le_bytes([
                slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
            ])
        } else {
            f64::from_be_bytes([
                slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
            ])
        }
    };
