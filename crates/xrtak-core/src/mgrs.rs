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
        let phi = lat_deg.to_radians();
        let lam = lon_deg.to_radians();
        let lam0 = gzd.central_meridian_deg().to_radians();

        let mut dlam = lam - lam0;
        while dlam > PI {
            dlam -= 2.0 * PI;
        }
        while dlam < -PI {
            dlam += 2.0 * PI;
        }

        let sin_p = phi.sin();
        let cos_p = phi.cos();
        let tan_p = phi.tan();

        let n = WGS84_A / (1.0 - WGS84_E2 * sin_p * sin_p).sqrt();
        let t = tan_p * tan_p;
        let c = WGS84_EP2 * cos_p * cos_p;
        let a = cos_p * dlam;

        // Meridional arc distance M from equator.
        let m = WGS84_A
            * ((1.0 - WGS84_E2 / 4.0 - 3.0 * WGS84_E2.powi(2) / 64.0 - 5.0 * WGS84_E2.powi(3) / 256.0)
                * phi
                - (3.0 * WGS84_E2 / 8.0 + 3.0 * WGS84_E2.powi(2) / 32.0 + 45.0 * WGS84_E2.powi(3) / 1024.0)
                    * (2.0 * phi).sin()
                + (15.0 * WGS84_E2.powi(2) / 256.0 + 45.0 * WGS84_E2.powi(3) / 1024.0) * (4.0 * phi).sin()
                - (35.0 * WGS84_E2.powi(3) / 3072.0) * (6.0 * phi).sin());

        let easting = FALSE_EASTING
            + UTM_K0
                * n
                * (a + (1.0 - t + c) * a.powi(3) / 6.0
                    + (5.0 - 18.0 * t + t * t + 72.0 * c - 58.0 * WGS84_EP2) * a.powi(5) / 120.0);

        let mut northing = UTM_K0
            * (m + n
                * tan_p
                * (a.powi(2) / 2.0
                    + (5.0 - t + 9.0 * c + 4.0 * c * c) * a.powi(4) / 24.0
                    + (61.0 - 58.0 * t + t * t + 600.0 * c - 330.0 * WGS84_EP2) * a.powi(6) / 720.0));

        let is_northern = lat_deg >= 0.0;
        if !is_northern {
            northing += FALSE_NORTHING_SOUTH;
        }

        Ok(Self {
            zone: gzd.zone,
            band: gzd.band,
            easting,
            northing,
            is_northern,
        })
    }

    /// Inverse projection: convert UTM coordinate back to Geodetic latitude / longitude.
    pub fn to_geodetic(&self) -> Result<GeodeticCoord, MgrsError> {
        let x = self.easting - FALSE_EASTING;
        let y = if self.is_northern {
            self.northing
        } else {
            self.northing - FALSE_NORTHING_SOUTH
        };

        let e1 = (1.0 - (1.0 - WGS84_E2).sqrt()) / (1.0 + (1.0 - WGS84_E2).sqrt());
        let m = y / UTM_K0;
        let mu = m
            / (WGS84_A
                * (1.0
                    - WGS84_E2 / 4.0
                    - 3.0 * WGS84_E2.powi(2) / 64.0
                    - 5.0 * WGS84_E2.powi(3) / 256.0));

        let phi1 = mu
            + (3.0 * e1 / 2.0 - 27.0 * e1.powi(3) / 32.0) * (2.0 * mu).sin()
            + (21.0 * e1.powi(2) / 16.0 - 55.0 * e1.powi(4) / 32.0) * (4.0 * mu).sin()
            + (151.0 * e1.powi(3) / 96.0) * (6.0 * mu).sin()
            + (1097.0 * e1.powi(4) / 512.0) * (8.0 * mu).sin();

        let sin_p1 = phi1.sin();
        let cos_p1 = phi1.cos();
        let tan_p1 = phi1.tan();

        let n1 = WGS84_A / (1.0 - WGS84_E2 * sin_p1 * sin_p1).sqrt();
        let r1 = WGS84_A * (1.0 - WGS84_E2) / (1.0 - WGS84_E2 * sin_p1 * sin_p1).powf(1.5);
        let d = x / (n1 * UTM_K0);

        let t1 = tan_p1 * tan_p1;
        let c1 = WGS84_EP2 * cos_p1 * cos_p1;

        let lat_rad = phi1
            - (n1 * tan_p1 / r1)
                * (d.powi(2) / 2.0
                    - (5.0 + 3.0 * t1 + 10.0 * c1 - 4.0 * c1 * c1 - 9.0 * WGS84_EP2) * d.powi(4) / 24.0
                    + (61.0 + 90.0 * t1 + 298.0 * c1 + 45.0 * t1 * t1
                        - 252.0 * WGS84_EP2
                        - 3.0 * c1 * c1)
                        * d.powi(6)
                        / 720.0);

        let cm_deg = f64::from(self.zone) * 6.0 - 183.0;
        let lon_rad = cm_deg.to_radians()
            + (d - (1.0 + 2.0 * t1 + c1) * d.powi(3) / 6.0
                + (5.0 - 2.0 * c1 + 28.0 * t1 - 3.0 * c1 * c1 + 8.0 * WGS84_EP2 + 24.0 * t1 * t1)
                    * d.powi(5)
                    / 120.0)
                / cos_p1;

        GeodeticCoord::new(lat_rad.to_degrees(), lon_rad.to_degrees())
    }

    /// Convert UTM coordinate to MGRS coordinate.
    pub fn to_mgrs(&self, precision: MgrsPrecision) -> Result<MgrsCoord, MgrsError> {
        let gzd = GridZoneDesignator::new(self.zone, self.band)?;
        let square = HundredKmSquare::from_utm(self.zone, self.easting, self.northing)?;

        let e_rem = (self.easting.rem_euclid(100_000.0)).floor() as u32;
        let n_rem = (self.northing.rem_euclid(100_000.0)).floor() as u32;

        MgrsCoord::new(gzd, square, e_rem, n_rem, precision)
    }
}

/// Military Grid Reference System (MGRS) coordinate reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MgrsCoord {
    /// Grid Zone Designator (zone + latitude band).
    pub gzd: GridZoneDesignator,
    /// 100,000-meter square identifier.
    pub square: HundredKmSquare,
    /// Easting within the 100k square (0..=99999).
    pub easting: u32,
    /// Northing within the 100k square (0..=99999).
    pub northing: u32,
    /// Precision resolution level.
    pub precision: MgrsPrecision,
}

impl MgrsCoord {
    /// Construct a verified MGRS coordinate.
    pub fn new(
        gzd: GridZoneDesignator,
        square: HundredKmSquare,
        easting: u32,
        northing: u32,
        precision: MgrsPrecision,
    ) -> Result<Self, MgrsError> {
        if easting >= 100_000 {
            return Err(MgrsError::InvalidFormat(format!("easting {easting} >= 100,000")));
        }
        if northing >= 100_000 {
            return Err(MgrsError::InvalidFormat(format!("northing {northing} >= 100,000")));
        }
        Ok(Self {
            gzd,
            square,
            easting,
            northing,
            precision,
        })
    }

    /// Parse an MGRS alphanumeric string into a coordinate.
    /// Handles compact ("31UDQ4825111943") or whitespace/hyphen separated ("31U DQ 48251 11943").
    pub fn parse(input: &str) -> Result<Self, MgrsError> {
        let clean: String = input
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect();

        if clean.len() < 3 {
            return Err(MgrsError::InvalidFormat(format!("input '{input}' is too short")));
        }

        // Zone: 1 or 2 digits
        let (zone_str, rem) = if clean.chars().take(2).all(|c| c.is_ascii_digit()) {
            clean.split_at(2)
        } else if clean.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            clean.split_at(1)
        } else {
            return Err(MgrsError::InvalidFormat("missing zone digits".to_string()));
        };

        let zone: u8 = zone_str
            .parse()
            .map_err(|_| MgrsError::InvalidUtmZone(0))?;

        if rem.is_empty() {
            return Err(MgrsError::InvalidFormat("missing latitude band".to_string()));
        }

        let band = rem.chars().next().unwrap();
        let gzd = GridZoneDesignator::new(zone, band)?;
        let rem2 = &rem[band.len_utf8()..];

        if rem2.len() < 2 {
            return Err(MgrsError::InvalidFormat("missing 100k square identifier".to_string()));
        }

        let mut chars = rem2.chars();
        let col = chars.next().unwrap();
        let row = chars.next().unwrap();
        let square = HundredKmSquare::new(col, row)?;
        let num_str = chars.as_str();

        if !num_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(MgrsError::InvalidFormat("non-numeric coordinate digits".to_string()));
        }

        if num_str.len() % 2 != 0 {
            return Err(MgrsError::InvalidFormat(format!(
                "odd number of coordinate digits ({})",
                num_str.len()
            )));
        }

        let digits_per_coord = num_str.len() / 2;
        if digits_per_coord > 5 {
            return Err(MgrsError::InvalidPrecision(digits_per_coord));
        }

        let precision = MgrsPrecision::from_digits_per_coord(digits_per_coord)?;

        let (easting, northing) = if digits_per_coord == 0 {
            (0, 0)
        } else {
            let (e_str, n_str) = num_str.split_at(digits_per_coord);
            let raw_e: u32 = e_str.parse().unwrap_or(0);
            let raw_n: u32 = n_str.parse().unwrap_or(0);
            let scale = 10u32.pow((5 - digits_per_coord) as u32);
            (raw_e * scale, raw_n * scale)
        };

        Self::new(gzd, square, easting, northing, precision)
    }

    /// Format as compact string (e.g. "31UDQ4825111943").
    pub fn to_compact_string(&self) -> String {
        let p = self.precision.digits_per_coord();
        if p == 0 {
            format!("{}{}{}{}", self.gzd.zone, self.gzd.band, self.square.col, self.square.row)
        } else {
            let divisor = 10u32.pow((5 - p) as u32);
            let e = self.easting / divisor;
            let n = self.northing / divisor;
            format!(
                "{}{}{}{}{:0width$}{:0width$}",
                self.gzd.zone,
                self.gzd.band,
                self.square.col,
                self.square.row,
