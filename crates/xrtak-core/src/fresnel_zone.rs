//! Line-of-sight Fresnel zone clearance and RF link budget calculator.
//!
//! Evaluates Fresnel zone radii, Earth curvature bulge, obstacle clearances,
//! knife-edge diffraction loss, and total RF link budget margins.

use serde::{Deserialize, Serialize};

/// Speed of light in vacuum (meters per second).
pub const SPEED_OF_LIGHT_MPS: f64 = 299_792_458.0;

/// Mean Earth radius in meters (WGS-84 reference).
pub const DEFAULT_EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Standard atmospheric refraction k-factor (4/3 Earth model).
pub const DEFAULT_K_FACTOR: f64 = 4.0 / 3.0;

/// Clearance classification of the Fresnel zone relative to obstacles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearanceStatus {
    /// Over 100% first Fresnel zone clearance (completely open).
    Clear,
    /// 60% to 100% clearance (sub-clear, negligible diffraction loss).
    SubClear,
    /// 0% to 60% clearance (grazing path, diffraction attenuation occurs).
    Grazing,
    /// LOS ray is blocked or penetrated by terrain/obstacle.
    Obstructed,
}

/// Overall link performance categorization based on link margin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkQuality {
    /// Link margin >= 20 dB (robust tactical comms).
    Excellent,
    /// Link margin >= 10 dB (reliable link).
    Good,
    /// Link margin >= 0 dB (marginal link, prone to fading).
    Marginal,
    /// Link margin < 0 dB (severed / unreachable).
    Severed,
}

/// Obstacle point along the path between transmitter and receiver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObstaclePoint {
    /// Distance from transmitter in meters.
    pub distance_m: f64,
    /// Natural terrain elevation in meters AMSL.
    pub terrain_elevation_m: f64,
    /// Structure or tree canopy height above terrain in meters.
    pub structure_height_m: f64,
    /// Descriptive name or identifier of obstacle.
    pub name: String,
}

impl ObstaclePoint {
    /// Total peak elevation including structure height above sea level.
    pub fn total_elevation_m(&self) -> f64 {
        self.terrain_elevation_m + self.structure_height_m
    }
}

/// Evaluation of a single point along the propagation path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FresnelPointAnalysis {
    /// Distance from transmitter in meters.
    pub distance_m: f64,
    /// Direct line-of-sight ray elevation in meters AMSL.
    pub los_elevation_m: f64,
    /// Total obstacle elevation in meters AMSL.
    pub obstacle_elevation_m: f64,
    /// Earth curvature bulge elevation correction in meters.
    pub earth_bulge_m: f64,
    /// Net clearance between LOS ray and effective obstacle peak.
    pub net_clearance_m: f64,
    /// Radius of the Fresnel zone at this point in meters.
    pub fresnel_radius_m: f64,
    /// Ratio of clearance to Fresnel zone radius.
    pub clearance_ratio: f64,
    /// Clearance status classification.
    pub status: ClearanceStatus,
}
