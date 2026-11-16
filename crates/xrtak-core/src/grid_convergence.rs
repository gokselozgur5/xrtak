//! Grid convergence angle and scale factor calculator for UTM and UPS.
//!
//! Provides geodetic calculations on the WGS-84 ellipsoid for:
//! - Universal Transverse Mercator (UTM) zones (80S to 84N)
//! - Universal Polar Stereographic (UPS) North (>= 80N) and South (<= -80S)
//! - Grid convergence (angle between True North and Grid North)
//! - Point scale factor (ratio of grid distance to ground distance)
//! - True and Grid azimuth / distance conversions

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// WGS-84 semi-major axis in meters.
pub const WGS84_A: f64 = 6_378_137.0;

/// WGS-84 reciprocal flattening.
pub const WGS84_RECIPROCAL_F: f64 = 298.257_223_563;

/// WGS-84 first eccentricity squared: e^2 = 2f - f^2.
pub const WGS84_E2: f64 = 0.006_694_379_990_141_316_5;

/// WGS-84 second eccentricity squared: e'^2 = e^2 / (1 - e^2).
pub const WGS84_EP2: f64 = 0.006_739_496_742_276_434;

/// UTM central meridian scale factor.
pub const UTM_K0: f64 = 0.9996;

/// UPS pole scale factor.
pub const UPS_K0: f64 = 0.9940;

/// Grid projection system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridSystem {
    /// Universal Transverse Mercator with zone (1..=60) and hemisphere.
    Utm { zone: u8, north: bool },
    /// Universal Polar Stereographic (North polar cap, >= 80N).
    UpsNorth,
    /// Universal Polar Stereographic (South polar cap, <= -80S).
    UpsSouth,
}

/// Errors during grid convergence and scale factor computation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridError {
    /// Latitude exceeds [-90.0, 90.0] degrees.
    InvalidLatitude(String),
    /// Longitude exceeds [-180.0, 180.0] degrees.
    InvalidLongitude(String),
    /// UTM zone is outside 1..=60.
    InvalidUtmZone(u8),
    /// Latitude is outside the valid UTM band (-80.0S to 84.0N).
    LatitudeOutOfUtmRange(String),
    /// Latitude is outside the valid UPS band.
    LatitudeOutOfUpsRange(String),
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLatitude(msg) => write!(f, "invalid latitude: {}", msg),
            Self::InvalidLongitude(msg) => write!(f, "invalid longitude: {}", msg),
            Self::InvalidUtmZone(z) => write!(f, "invalid UTM zone: {} (must be 1..=60)", z),
            Self::LatitudeOutOfUtmRange(msg) => write!(f, "latitude out of UTM range: {}", msg),
            Self::LatitudeOutOfUpsRange(msg) => write!(f, "latitude out of UPS range: {}", msg),
        }
    }
}

impl std::error::Error for GridError {}

/// Result of grid convergence and scale factor calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridSolution {
    /// Grid projection system (UTM zone or UPS polar aspect).
    pub system: GridSystem,
    /// Grid convergence angle in decimal degrees (Grid North - True North).
    /// Positive when Grid North is clockwise (East) of True North.
    pub convergence_deg: f64,
    /// Grid convergence angle in radians.
    pub convergence_rad: f64,
    /// Point scale factor k (grid length / ellipsoidal surface length).
    pub scale_factor: f64,
    /// Central meridian longitude in decimal degrees.
    pub central_meridian_deg: f64,
}

/// Determine the standard UTM zone number (1..=60) for a given longitude in degrees.
pub fn utm_zone_from_lon(lon_deg: f64) -> u8 {
    let mut normalized_lon = lon_deg;
    while normalized_lon < -180.0 {
        normalized_lon += 360.0;
    }
    while normalized_lon >= 180.0 {
        normalized_lon -= 360.0;
    }
    let zone = ((normalized_lon + 180.0) / 6.0).floor() as i32 + 1;
    zone.clamp(1, 60) as u8
}

/// Determine the central meridian longitude in decimal degrees for a UTM zone (1..=60).
pub fn utm_central_meridian_deg(zone: u8) -> Result<f64, GridError> {
    if !(1..=60).contains(&zone) {
        return Err(GridError::InvalidUtmZone(zone));
    }
    Ok(f64::from(zone) * 6.0 - 183.0)
}

/// Compute UTM grid convergence angle and point scale factor for given latitude and longitude.
pub fn calculate_utm(
    lat_deg: f64,
    lon_deg: f64,
    zone_override: Option<u8>,
) -> Result<GridSolution, GridError> {
    if !(-90.0..=90.0).contains(&lat_deg) {
        return Err(GridError::InvalidLatitude(format!("{lat_deg}")));
    }
    if !(-180.0..=180.0).contains(&lon_deg) {
        return Err(GridError::InvalidLongitude(format!("{lon_deg}")));
    }
    if lat_deg < -80.0 || lat_deg > 84.0 {
        return Err(GridError::LatitudeOutOfUtmRange(format!(
            "latitude {lat_deg} is outside UTM coverage (-80 to 84)"
        )));
    }

    let zone = match zone_override {
        Some(z) => {
            if !(1..=60).contains(&z) {
                return Err(GridError::InvalidUtmZone(z));
            }
            z
        }
        None => utm_zone_from_lon(lon_deg),
    };

    let cm_deg = utm_central_meridian_deg(zone)?;
    let mut dlam_deg = lon_deg - cm_deg;
    while dlam_deg > 180.0 {
        dlam_deg -= 360.0;
    }
    while dlam_deg < -180.0 {
        dlam_deg += 360.0;
    }

    let phi = lat_deg.to_radians();
    let dlam = dlam_deg.to_radians();

    let sin_p = phi.sin();
    let cos_p = phi.cos();
    let tan_p = phi.tan();
    let eta2 = WGS84_EP2 * cos_p * cos_p;

    // Series expansion for Transverse Mercator convergence (Snyder eq. 8-16 / DMA TM 8358.1):
    let t1 = dlam * sin_p;
    let t2 = (dlam.powi(3) / 3.0) * sin_p * cos_p.powi(2) * (1.0 + 3.0 * eta2 + 2.0 * eta2.powi(2));
    let t3 = (dlam.powi(5) / 15.0)
        * sin_p
        * cos_p.powi(4)
        * (2.0 - tan_p.powi(2) + 15.0 * eta2 + 35.0 * eta2.powi(2) - 15.0 * tan_p.powi(2) * eta2);
    let gamma_rad = t1 + t2 + t3;
    let gamma_deg = gamma_rad.to_degrees();

    // Scale factor k (Snyder eq. 8-15 / DMA TM 8358.1):
    let k_term1 = 1.0;
    let k_term2 = 0.5 * dlam.powi(2) * cos_p.powi(2) * (1.0 + eta2);
    let k_term3 = (dlam.powi(4) / 24.0)
        * cos_p.powi(4)
        * (5.0 - 4.0 * tan_p.powi(2) + 14.0 * eta2 + 13.0 * eta2.powi(2)
            - 28.0 * tan_p.powi(2) * eta2);
    let scale_factor = UTM_K0 * (k_term1 + k_term2 + k_term3);

    Ok(GridSolution {
        system: GridSystem::Utm {
            zone,
            north: lat_deg >= 0.0,
        },
        convergence_deg: gamma_deg,
        convergence_rad: gamma_rad,
        scale_factor,
        central_meridian_deg: cm_deg,
    })
}

/// Compute UPS grid convergence angle and point scale factor for polar latitudes.
pub fn calculate_ups(lat_deg: f64, lon_deg: f64) -> Result<GridSolution, GridError> {
    if !(-90.0..=90.0).contains(&lat_deg) {
        return Err(GridError::InvalidLatitude(format!("{lat_deg}")));
    }
    if !(-180.0..=180.0).contains(&lon_deg) {
        return Err(GridError::InvalidLongitude(format!("{lon_deg}")));
    }

    let is_north = lat_deg >= 80.0;
    let is_south = lat_deg <= -80.0;

    if !is_north && !is_south {
        return Err(GridError::LatitudeOutOfUpsRange(format!(
            "latitude {lat_deg} is outside UPS coverage (>=84N or <=-80S)"
        )));
    }

    let e = WGS84_E2.sqrt();
    let phi_abs = lat_deg.abs().to_radians();

    // Point scale factor for ellipsoidal polar stereographic (Snyder eq. 21-17, DMA TM 8358.1):
    let scale_factor = if phi_abs >= (PI / 2.0 - 1e-12) {
        UPS_K0
    } else {
        let sin_p = phi_abs.sin();
        let cos_p = phi_abs.cos();
        let t = (PI / 4.0 - phi_abs / 2.0).tan()
            / (((1.0 - e * sin_p) / (1.0 + e * sin_p)).powf(e / 2.0));
        let m = cos_p / (1.0 - WGS84_E2 * sin_p * sin_p).sqrt();
        let denom = (((1.0 + e).powf(1.0 + e)) * ((1.0 - e).powf(1.0 - e))).sqrt();
        let rho = 2.0 * WGS84_A * UPS_K0 * t / denom;
        rho / (WGS84_A * m)
    };

    let (system, convergence_deg, cm_deg) = if is_north {
        // North UPS: Grid North along 180th meridian, gamma = lon
        let mut conv = lon_deg;
        while conv > 180.0 {
            conv -= 360.0;
        }
        while conv < -180.0 {
            conv += 360.0;
        }
        (GridSystem::UpsNorth, conv, 0.0)
    } else {
        // South UPS: Grid North along Prime Meridian, gamma = -lon
        let mut conv = -lon_deg;
        while conv > 180.0 {
            conv -= 360.0;
        }
        while conv < -180.0 {
            conv += 360.0;
        }
        (GridSystem::UpsSouth, conv, 0.0)
    };

    Ok(GridSolution {
        system,
        convergence_deg,
        convergence_rad: convergence_deg.to_radians(),
        scale_factor,
        central_meridian_deg: cm_deg,
    })
}

/// Automatically select between UTM and UPS based on latitude, and compute grid solution.
pub fn calculate_grid_convergence(lat_deg: f64, lon_deg: f64) -> Result<GridSolution, GridError> {
    if lat_deg >= 80.0 || lat_deg <= -80.0 {
        calculate_ups(lat_deg, lon_deg)
    } else {
        calculate_utm(lat_deg, lon_deg, None)
    }
}

/// Convert true azimuth (degrees from True North) to grid azimuth (degrees from Grid North).
///
/// Formula: Grid Azimuth = True Azimuth - Convergence Angle.
pub fn true_to_grid_azimuth(true_azimuth_deg: f64, convergence_deg: f64) -> f64 {
    let mut grid_azimuth = true_azimuth_deg - convergence_deg;
    while grid_azimuth < 0.0 {
        grid_azimuth += 360.0;
    }
    while grid_azimuth >= 360.0 {
        grid_azimuth -= 360.0;
    }
    grid_azimuth
}

/// Convert grid azimuth (degrees from Grid North) to true azimuth (degrees from True North).
///
/// Formula: True Azimuth = Grid Azimuth + Convergence Angle.
pub fn grid_to_true_azimuth(grid_azimuth_deg: f64, convergence_deg: f64) -> f64 {
    let mut true_azimuth = grid_azimuth_deg + convergence_deg;
    while true_azimuth < 0.0 {
        true_azimuth += 360.0;
    }
    while true_azimuth >= 360.0 {
        true_azimuth -= 360.0;
    }
    true_azimuth
}

/// Apply scale factor to ground distance to get projected grid distance in meters.
pub fn ground_to_grid_distance(ground_distance_m: f64, scale_factor: f64) -> f64 {
    ground_distance_m * scale_factor
}

/// Convert projected grid distance in meters back to ellipsoidal ground distance.
pub fn grid_to_ground_distance(grid_distance_m: f64, scale_factor: f64) -> f64 {
    if scale_factor.abs() < 1e-12 {
        ground_to_grid_distance(grid_distance_m, 1.0)
    } else {
        grid_distance_m / scale_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utm_zone_and_cm() {
        assert_eq!(utm_zone_from_lon(-75.0), 18);
        assert_eq!(utm_central_meridian_deg(18).unwrap(), -75.0);
        assert_eq!(utm_zone_from_lon(0.0), 31);
        assert_eq!(utm_central_meridian_deg(31).unwrap(), 3.0);
    }

    #[test]
    fn test_utm_central_meridian_convergence_and_scale() {
        let sol = calculate_utm(30.0, -75.0, Some(18)).unwrap();
        assert!((sol.convergence_deg).abs() < 1e-10);
        assert!((sol.scale_factor - UTM_K0).abs() < 1e-6);
    }

    #[test]
    fn test_utm_off_meridian_convergence() {
        let sol = calculate_utm(30.0, -72.0, Some(18)).unwrap();
        // At 3 deg east, convergence is approx dlam * sin(lat) ~ 3 * 0.5 = 1.5 deg
        assert!((sol.convergence_deg - 1.501).abs() < 0.01);
