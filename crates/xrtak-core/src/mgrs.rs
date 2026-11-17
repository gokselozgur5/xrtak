//! Military Grid Reference System (MGRS) precision coordinate converter.
//!
//! Provides bidirectional conversion between:
//! - Geodetic coordinates (WGS-84 latitude / longitude / elevation)
//! - Universal Transverse Mercator (UTM) coordinates
//! - Military Grid Reference System (MGRS) alphanumeric grid references
//!
//! Supports variable precision levels (1m, 10m, 100m, 1km, 10km, 100km),
//! bounding box calculation, center coordinate resolution, and formatting.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// WGS-84 semi-major axis in meters.
pub const WGS84_A: f64 = 6_378_137.0;

/// WGS-84 reciprocal flattening (1 / f).
pub const WGS84_RECIPROCAL_F: f64 = 298.257_223_563;

/// WGS-84 flattening f.
pub const WGS84_F: f64 = 1.0 / WGS84_RECIPROCAL_F;

/// WGS-84 first eccentricity squared (e^2 = 2f - f^2).
pub const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F;

/// WGS-84 second eccentricity squared (e'^2 = e^2 / (1 - e^2)).
pub const WGS84_EP2: f64 = WGS84_E2 / (1.0 - WGS84_E2);

/// UTM central scale factor.
pub const UTM_K0: f64 = 0.9996;

/// UTM false easting in meters.
pub const FALSE_EASTING: f64 = 500_000.0;

/// UTM false northing for southern hemisphere in meters.
pub const FALSE_NORTHING_SOUTH: f64 = 10_000_000.0;

/// MGRS latitude bands covering -80.0 to 84.0 degrees latitude.
pub const LATITUDE_BANDS: [(char, f64, f64); 20] = [
    ('C', -80.0, -72.0),
    ('D', -72.0, -64.0),
    ('E', -64.0, -56.0),
    ('F', -56.0, -48.0),
    ('G', -48.0, -40.0),
    ('H', -40.0, -32.0),
    ('J', -32.0, -24.0),
    ('K', -24.0, -16.0),
    ('L', -16.0, -8.0),
    ('M', -8.0, 0.0),
    ('N', 0.0, 8.0),
    ('P', 8.0, 16.0),
    ('Q', 16.0, 24.0),
    ('R', 24.0, 32.0),
    ('S', 32.0, 40.0),
    ('T', 40.0, 48.0),
    ('U', 48.0, 56.0),
    ('V', 56.0, 64.0),
    ('W', 64.0, 72.0),
    ('X', 72.0, 84.0),
];

/// 100,000-meter square column letters for 3 repeating zone sets.
pub const COL_SETS: [[char; 8]; 3] = [
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'],
    ['J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R'],
    ['S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
];

/// 100,000-meter square row letters (repeating every 2,000,000 meters).
pub const ROW_LETTERS: [char; 20] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V',
];

/// Errors during MGRS / UTM / Geodetic conversion and parsing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MgrsError {
    /// Latitude exceeds [-80.0, 84.0] degrees for UTM coverage.
    InvalidLatitude(f64),
    /// Longitude exceeds [-180.0, 180.0] degrees.
    InvalidLongitude(f64),
    /// UTM zone is outside 1..=60.
    InvalidUtmZone(u8),
    /// Invalid latitude band letter.
    InvalidLatitudeBand(char),
    /// Invalid 100k square identifier.
    Invalid100kSquare(String),
    /// Precision digit count per coordinate is outside 0..=5.
    InvalidPrecision(usize),
    /// MGRS string format is invalid or malformed.
    InvalidFormat(String),
    /// Calculated northing is out of range for the grid zone.
    NorthingOutOfRange(f64),
}

impl std::fmt::Display for MgrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLatitude(lat) => write!(f, "latitude {lat} out of range [-80.0, 84.0]"),
            Self::InvalidLongitude(lon) => write!(f, "longitude {lon} out of range [-180.0, 180.0]"),
            Self::InvalidUtmZone(z) => write!(f, "invalid UTM zone {z}, must be 1..=60"),
            Self::InvalidLatitudeBand(b) => write!(f, "invalid MGRS latitude band '{b}'"),
            Self::Invalid100kSquare(sq) => write!(f, "invalid 100k square '{sq}'"),
            Self::InvalidPrecision(p) => write!(f, "invalid precision digits {p}, must be 0..=5"),
            Self::InvalidFormat(msg) => write!(f, "invalid MGRS format: {msg}"),
            Self::NorthingOutOfRange(n) => write!(f, "northing {n} out of range for zone"),
        }
    }
}

impl std::error::Error for MgrsError {}

/// MGRS coordinate grid resolution precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MgrsPrecision {
    /// 100 km grid precision (0 digits: e.g. "31UCT").
    HundredKilometers,
    /// 10 km grid precision (2 digits: 1 easting, 1 northing).
    TenKilometers,
    /// 1 km grid precision (4 digits: 2 easting, 2 northing).
    OneKilometer,
    /// 100 m grid precision (6 digits: 3 easting, 3 northing).
    HundredMeters,
    /// 10 m grid precision (8 digits: 4 easting, 4 northing).
    TenMeters,
    /// 1 m grid precision (10 digits: 5 easting, 5 northing).
    OneMeter,
}

impl MgrsPrecision {
    /// Construct precision from digits per coordinate (0 to 5).
    pub fn from_digits_per_coord(digits: usize) -> Result<Self, MgrsError> {
        match digits {
            0 => Ok(Self::HundredKilometers),
            1 => Ok(Self::TenKilometers),
            2 => Ok(Self::OneKilometer),
            3 => Ok(Self::HundredMeters),
            4 => Ok(Self::TenMeters),
            5 => Ok(Self::OneMeter),
            other => Err(MgrsError::InvalidPrecision(other)),
        }
    }

    /// Number of digits used for easting and northing in this precision level.
    pub fn digits_per_coord(&self) -> usize {
        match self {
            Self::HundredKilometers => 0,
            Self::TenKilometers => 1,
            Self::OneKilometer => 2,
            Self::HundredMeters => 3,
            Self::TenMeters => 4,
            Self::OneMeter => 5,
        }
    }

    /// Linear grid cell resolution in meters.
    pub fn resolution_meters(&self) -> f64 {
        match self {
            Self::HundredKilometers => 100_000.0,
            Self::TenKilometers => 10_000.0,
            Self::OneKilometer => 1_000.0,
            Self::HundredMeters => 100.0,
            Self::TenMeters => 10.0,
            Self::OneMeter => 1.0,
        }
    }
}

/// Grid Zone Designator (GZD) composed of UTM zone (1..=60) and latitude band ('C'..='X').
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridZoneDesignator {
    /// UTM Zone number (1 to 60).
    pub zone: u8,
    /// MGRS latitude band letter ('C' to 'X', excluding 'I' and 'O').
    pub band: char,
}

impl GridZoneDesignator {
    /// Create and validate a Grid Zone Designator.
    pub fn new(zone: u8, band: char) -> Result<Self, MgrsError> {
        if !(1..=60).contains(&zone) {
            return Err(MgrsError::InvalidUtmZone(zone));
        }
        let b = band.to_ascii_uppercase();
        if !LATITUDE_BANDS.iter().any(|(band_char, _, _)| *band_char == b) {
            return Err(MgrsError::InvalidLatitudeBand(band));
        }
        Ok(Self { zone, band: b })
    }

    /// Determine GZD from geodetic latitude and longitude in degrees.
    pub fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> Result<Self, MgrsError> {
        if !(-80.0..=84.0).contains(&lat_deg) {
            return Err(MgrsError::InvalidLatitude(lat_deg));
        }
        if !(-180.0..=180.0).contains(&lon_deg) {
            return Err(MgrsError::InvalidLongitude(lon_deg));
        }

        let mut norm_lon = lon_deg;
        while norm_lon < -180.0 {
            norm_lon += 360.0;
        }
        while norm_lon >= 180.0 {
            norm_lon -= 360.0;
        }

        let zone = (((norm_lon + 180.0) / 6.0).floor() as i32 + 1).clamp(1, 60) as u8;

        let band = LATITUDE_BANDS
            .iter()
            .find(|(_, min, max)| lat_deg >= *min && (lat_deg < *max || (lat_deg == 84.0 && *max == 84.0)))
            .map(|(b, _, _)| *b)
            .unwrap_or('X');

        Self::new(zone, band)
    }

    /// Check if this GZD is in the Northern Hemisphere.
    pub fn is_northern(&self) -> bool {
        self.band >= 'N'
    }

    /// Get latitude range [min_lat, max_lat] in degrees.
    pub fn latitude_bounds(&self) -> (f64, f64) {
        LATITUDE_BANDS
            .iter()
            .find(|(b, _, _)| *b == self.band)
            .map(|(_, min, max)| (*min, *max))
            .unwrap_or((0.0, 0.0))
    }

    /// Central meridian longitude in decimal degrees.
    pub fn central_meridian_deg(&self) -> f64 {
        f64::from(self.zone) * 6.0 - 183.0
    }
}

/// 100,000-meter square identification (two alphabetic characters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HundredKmSquare {
    /// Column letter ('A'..='Z', excluding 'I' and 'O').
    pub col: char,
    /// Row letter ('A'..='V', excluding 'I' and 'O').
    pub row: char,
}

impl HundredKmSquare {
    /// Construct and validate a 100k square identifier.
    pub fn new(col: char, row: char) -> Result<Self, MgrsError> {
        let c = col.to_ascii_uppercase();
        let r = row.to_ascii_uppercase();

        let valid_col = COL_SETS.iter().any(|set| set.contains(&c));
        let valid_row = ROW_LETTERS.contains(&r);

        if !valid_col || !valid_row {
            return Err(MgrsError::Invalid100kSquare(format!("{col}{row}")));
        }
        Ok(Self { col: c, row: r })
    }

    /// Determine 100k square identifier from UTM zone, easting, and northing.
    pub fn from_utm(zone: u8, easting: f64, northing: f64) -> Result<Self, MgrsError> {
        if !(1..=60).contains(&zone) {
            return Err(MgrsError::InvalidUtmZone(zone));
        }
        let set_idx = ((zone - 1) % 3) as usize;
        let col_num = (easting / 100_000.0).floor() as isize;
        if !(1..=8).contains(&col_num) {
            return Err(MgrsError::Invalid100kSquare(format!(
                "easting {easting} yields invalid column {col_num}"
            )));
        }
        let col = COL_SETS[set_idx][(col_num - 1) as usize];

        let row_offset = if zone % 2 == 0 { 5 } else { 0 };
        let row_num = (((northing / 100_000.0).floor() as isize + row_offset).rem_euclid(20)) as usize;
        let row = ROW_LETTERS[row_num];

        Self::new(col, row)
    }

    /// Compute base UTM easting and northing within a GZD for this 100k square.
    pub fn to_utm_offsets(&self, gzd: &GridZoneDesignator) -> Result<(f64, f64), MgrsError> {
        let set_idx = ((gzd.zone - 1) % 3) as usize;
        let col_pos = COL_SETS[set_idx]
            .iter()
            .position(|&c| c == self.col)
            .ok_or_else(|| MgrsError::Invalid100kSquare(format!("column {}", self.col)))?;
        let base_easting = (col_pos as f64 + 1.0) * 100_000.0;

        let row_pos = ROW_LETTERS
            .iter()
            .position(|&r| r == self.row)
            .ok_or_else(|| MgrsError::Invalid100kSquare(format!("row {}", self.row)))?;
        let row_offset = if gzd.zone % 2 == 0 { 5 } else { 0 };
        let base_northing_mod = ((row_pos as isize - row_offset).rem_euclid(20)) as f64 * 100_000.0;

        // Solve 2,000,000m cycle within the latitude band.
        let (min_lat, max_lat) = gzd.latitude_bounds();
        let is_north = gzd.is_northern();

        let mut best_northing = None;
        let mut min_diff = f64::MAX;

        for k in 0..6 {
            let candidate_n = k as f64 * 2_000_000.0 + base_northing_mod;
            if is_north && !(0.0..=9_600_000.0).contains(&candidate_n) {
                continue;
            }
            if !is_north && !(1_000_000.0..=10_000_000.0).contains(&candidate_n) {
                continue;
            }

            // Approximate latitude of candidate northing at zone center.
            let approx_lat = approximate_northing_to_lat(candidate_n, is_north);
            let diff = if approx_lat < min_lat {
                min_lat - approx_lat
            } else if approx_lat > max_lat {
                approx_lat - max_lat
            } else {
                0.0
            };

            if diff < min_diff {
                min_diff = diff;
                best_northing = Some(candidate_n);
                if diff == 0.0 {
                    break;
                }
            }
        }

        let base_northing = best_northing.ok_or(MgrsError::NorthingOutOfRange(base_northing_mod))?;
        Ok((base_easting, base_northing))
    }
}

/// Approximate inverse mapping from UTM northing to geodetic latitude for band disambiguation.
fn approximate_northing_to_lat(northing: f64, is_north: bool) -> f64 {
    let y = if is_north {
        northing
    } else {
        northing - FALSE_NORTHING_SOUTH
    };
    (y / (UTM_K0 * 111_132.954)).clamp(-80.0, 84.0)
}

/// Geodetic WGS-84 coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeodeticCoord {
    /// Latitude in decimal degrees [-90.0, 90.0].
    pub lat_deg: f64,
    /// Longitude in decimal degrees [-180.0, 180.0].
    pub lon_deg: f64,
    /// Optional ellipsoidal or orthometric altitude in meters.
    pub alt_m: Option<f64>,
}

impl GeodeticCoord {
    /// Construct a 2D geodetic coordinate.
    pub fn new(lat_deg: f64, lon_deg: f64) -> Result<Self, MgrsError> {
        if !(-90.0..=90.0).contains(&lat_deg) {
            return Err(MgrsError::InvalidLatitude(lat_deg));
        }
        if !(-180.0..=180.0).contains(&lon_deg) {
            return Err(MgrsError::InvalidLongitude(lon_deg));
        }
        Ok(Self {
            lat_deg,
            lon_deg,
            alt_m: None,
        })
    }

    /// Construct a 3D geodetic coordinate with altitude in meters.
    pub fn with_alt(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Result<Self, MgrsError> {
        let mut coord = Self::new(lat_deg, lon_deg)?;
        coord.alt_m = Some(alt_m);
        Ok(coord)
    }

    /// Convert geodetic coordinate to UTM coordinate.
    pub fn to_utm(&self) -> Result<UtmCoord, MgrsError> {
        UtmCoord::from_lat_lon(self.lat_deg, self.lon_deg)
    }

    /// Convert geodetic coordinate directly to MGRS coordinate at given precision.
    pub fn to_mgrs(&self, precision: MgrsPrecision) -> Result<MgrsCoord, MgrsError> {
        let utm = self.to_utm()?;
        utm.to_mgrs(precision)
    }
}

/// Universal Transverse Mercator (UTM) coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UtmCoord {
    /// UTM Zone number (1 to 60).
    pub zone: u8,
    /// MGRS latitude band letter ('C' to 'X').
    pub band: char,
    /// Easting in meters (typically 100,000 to 900,000).
    pub easting: f64,
    /// Northing in meters (0 to 10,000,000).
    pub northing: f64,
    /// True if Northern Hemisphere, false if Southern Hemisphere.
    pub is_northern: bool,
}

impl UtmCoord {
    /// Forward projection: convert latitude and longitude to UTM coordinate.
    pub fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> Result<Self, MgrsError> {
        let gzd = GridZoneDesignator::from_lat_lon(lat_deg, lon_deg)?;
