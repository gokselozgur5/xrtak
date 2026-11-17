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

/// Tactical RF transceiver hardware and link parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RfTransceiver {
    /// Transmitter output power in dBm.
    pub tx_power_dbm: f64,
    /// Transmitter antenna gain in dBi.
    pub tx_gain_dbi: f64,
    /// Transmitter cable and connector losses in dB.
    pub tx_loss_db: f64,
    /// Receiver antenna gain in dBi.
    pub rx_gain_dbi: f64,
    /// Receiver cable and connector losses in dB.
    pub rx_loss_db: f64,
    /// Receiver sensitivity threshold in dBm.
    pub rx_sensitivity_dbm: f64,
    /// Desired fade margin buffer in dB.
    pub fade_margin_db: f64,
}

impl Default for RfTransceiver {
    fn default() -> Self {
        Self {
            tx_power_dbm: 30.0,
            tx_gain_dbi: 2.15,
            tx_loss_db: 1.0,
            rx_gain_dbi: 2.15,
            rx_loss_db: 1.0,
            rx_sensitivity_dbm: -100.0,
            fade_margin_db: 10.0,
        }
    }
}

/// Configuration for Fresnel zone evaluation and propagation geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FresnelCalculatorConfig {
    /// Carrier frequency in Hertz.
    pub frequency_hz: f64,
    /// Effective Earth radius in meters.
    pub earth_radius_m: f64,
    /// Atmospheric refraction factor (typically 1.333 for standard atmosphere).
    pub k_factor: f64,
    /// Fresnel zone order (1 for first Fresnel zone).
    pub zone_number: u32,
    /// Additional miscellaneous path losses in dB (rain, foliage, etc.).
    pub misc_loss_db: f64,
}

impl Default for FresnelCalculatorConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 2_400_000_000.0, // 2.4 GHz ISM
            earth_radius_m: DEFAULT_EARTH_RADIUS_M,
            k_factor: DEFAULT_K_FACTOR,
            zone_number: 1,
            misc_loss_db: 0.0,
        }
    }
}

/// Comprehensive RF link budget and Fresnel clearance summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkBudgetResult {
    /// Carrier frequency in Hz.
    pub frequency_hz: f64,
    /// Total path distance in meters.
    pub total_distance_m: f64,
    /// Equivalent Isotropically Radiated Power (EIRP) in dBm.
    pub eirp_dbm: f64,
    /// Free Space Path Loss in dB.
    pub fspl_db: f64,
    /// Knife-edge diffraction attenuation in dB.
    pub diffraction_loss_db: f64,
    /// Miscellaneous losses in dB.
    pub misc_loss_db: f64,
    /// Total received power at receiver input in dBm.
    pub rx_power_dbm: f64,
    /// Calculated link margin in dB.
    pub link_margin_db: f64,
    /// Worst-case Fresnel clearance ratio along the link.
    pub worst_clearance_ratio: f64,
    /// Worst-case bottleneck obstruction point details.
    pub worst_point: Option<FresnelPointAnalysis>,
    /// Link operational status categorization.
    pub quality: LinkQuality,
}
