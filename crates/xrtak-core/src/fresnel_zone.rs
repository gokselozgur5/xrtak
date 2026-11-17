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

/// Computes signal wavelength in meters from frequency in Hertz.
pub fn compute_wavelength(frequency_hz: f64) -> f64 {
    if frequency_hz <= 0.0 {
        return 0.0;
    }
    SPEED_OF_LIGHT_MPS / frequency_hz
}

/// Calculates radius of nth Fresnel zone in meters at distances d1 and d2.
pub fn compute_fresnel_radius(n: u32, d1_m: f64, d2_m: f64, frequency_hz: f64) -> f64 {
    let total_d = d1_m + d2_m;
    if total_d <= 1e-4 || d1_m <= 0.0 || d2_m <= 0.0 || n == 0 {
        return 0.0;
    }
    let lambda = compute_wavelength(frequency_hz);
    if lambda <= 0.0 {
        return 0.0;
    }
    ((n as f64 * lambda * d1_m * d2_m) / total_d).sqrt()
}

/// Computes Earth curvature bulge elevation in meters along a line of sight.
pub fn compute_earth_bulge(d1_m: f64, d2_m: f64, k_factor: f64, earth_radius_m: f64) -> f64 {
    let effective_r = k_factor * earth_radius_m;
    if effective_r <= 0.0 || d1_m <= 0.0 || d2_m <= 0.0 {
        return 0.0;
    }
    (d1_m * d2_m) / (2.0 * effective_r)
}

/// Computes Free-Space Path Loss (FSPL) in decibels.
pub fn compute_fspl(distance_m: f64, frequency_hz: f64) -> f64 {
    let d = distance_m.max(1.0);
    let lambda = compute_wavelength(frequency_hz);
    if lambda <= 0.0 {
        return 0.0;
    }
    20.0 * (4.0 * std::f64::consts::PI * d / lambda).log10()
}

/// Computes dimensionless Fresnel-Kirchhoff knife-edge diffraction parameter v.
///
/// obstacle_penetration_m is the height of obstacle above the LOS ray (positive = blockage).
pub fn compute_diffraction_param_v(
    obstacle_penetration_m: f64,
    d1_m: f64,
    d2_m: f64,
    frequency_hz: f64,
) -> f64 {
    let total_d = d1_m + d2_m;
    if total_d <= 1e-4 || d1_m <= 0.0 || d2_m <= 0.0 {
        return -1.0;
    }
    let lambda = compute_wavelength(frequency_hz);
    if lambda <= 0.0 {
        return -1.0;
    }
    let geometry_factor = (2.0 * total_d / (lambda * d1_m * d2_m)).sqrt();
    obstacle_penetration_m * geometry_factor
}

/// Computes Knife-Edge Diffraction Loss J(v) in dB using ITU-R P.526-15 approximation.
///
/// v is the dimensionless diffraction parameter.
/// If v <= -0.7, obstacle is outside 0.6 Fresnel clearance (0 dB loss).
pub fn compute_knife_edge_diffraction(v: f64) -> f64 {
    if v <= -0.7 {
        0.0
    } else {
        let diff = v - 0.1;
        let term = (diff.powi(2) + 1.0).sqrt() + diff;
        if term <= 0.0 {
            0.0
        } else {
            let loss = 6.9 + 20.0 * term.log10();
            loss.max(0.0)
        }
    }
}

/// Classifies clearance ratio into standard operational clearance categories.
pub fn classify_clearance_status(net_clearance_m: f64, ratio: f64) -> ClearanceStatus {
    if net_clearance_m <= 0.0 {
        ClearanceStatus::Obstructed
    } else if ratio < 0.6 {
        ClearanceStatus::Grazing
    } else if ratio < 1.0 {
        ClearanceStatus::SubClear
    } else {
        ClearanceStatus::Clear
    }
}

/// Evaluates clearance and Fresnel radius for a single obstacle point.
pub fn evaluate_point_clearance(
    obs: &ObstaclePoint,
    tx_alt_m: f64,
    rx_alt_m: f64,
    total_dist_m: f64,
    config: &FresnelCalculatorConfig,
) -> FresnelPointAnalysis {
    let d = total_dist_m.max(1.0);
    let d1 = obs.distance_m.clamp(0.0, d);
    let d2 = (d - d1).max(0.0);
    let los_alt = tx_alt_m + (d1 / d) * (rx_alt_m - tx_alt_m);
    let bulge = compute_earth_bulge(d1, d2, config.k_factor, config.earth_radius_m);
    let eff_obs_alt = obs.total_elevation_m() + bulge;
    let net_clear = los_alt - eff_obs_alt;
    let r = compute_fresnel_radius(config.zone_number, d1, d2, config.frequency_hz);
    let ratio = if r > 1e-4 { net_clear / r } else { 1.0 };
    FresnelPointAnalysis {
        distance_m: d1,
        los_elevation_m: los_alt,
        obstacle_elevation_m: obs.total_elevation_m(),
        earth_bulge_m: bulge,
        net_clearance_m: net_clear,
        fresnel_radius_m: r,
        clearance_ratio: ratio,
        status: classify_clearance_status(net_clear, ratio),
    }
}

/// Evaluates Fresnel clearance profile for all obstacles along a link.
pub fn analyze_fresnel_profile(
    tx_alt_m: f64,
    rx_alt_m: f64,
    total_dist_m: f64,
    obstacles: &[ObstaclePoint],
    config: &FresnelCalculatorConfig,
) -> Vec<FresnelPointAnalysis> {
    obstacles
        .iter()
        .map(|obs| evaluate_point_clearance(obs, tx_alt_m, rx_alt_m, total_dist_m, config))
        .collect()
}

/// Finds the worst-case obstruction and computes peak diffraction loss.
pub fn find_worst_obstruction(
    profile: &[FresnelPointAnalysis],
    total_dist_m: f64,
    frequency_hz: f64,
) -> (f64, Option<FresnelPointAnalysis>, f64) {
    let mut min_ratio = 1.0f64;
    let mut worst_point: Option<FresnelPointAnalysis> = None;
    let mut max_diff_loss = 0.0f64;

    for pt in profile {
        if pt.clearance_ratio < min_ratio {
            min_ratio = pt.clearance_ratio;
            worst_point = Some(pt.clone());
        }
        let d1 = pt.distance_m;
        let d2 = (total_dist_m - d1).max(0.0);
        let pen = -pt.net_clearance_m;
        let v = compute_diffraction_param_v(pen, d1, d2, frequency_hz);
        let loss = compute_knife_edge_diffraction(v);
        if loss > max_diff_loss {
            max_diff_loss = loss;
        }
    }
    (min_ratio, worst_point, max_diff_loss)
}

/// Categorizes overall tactical link performance based on link margin.
pub fn classify_link_quality(link_margin_db: f64) -> LinkQuality {
    if link_margin_db >= 20.0 {
        LinkQuality::Excellent
    } else if link_margin_db >= 10.0 {
        LinkQuality::Good
    } else if link_margin_db >= 0.0 {
        LinkQuality::Marginal
    } else {
        LinkQuality::Severed
    }
}

/// Computes EIRP, received signal power, and fade-margin-adjusted link margin.
pub fn compute_rx_power_and_margin(
    t: &RfTransceiver,
    fspl: f64,
    diff_loss: f64,
    misc_loss: f64,
) -> (f64, f64, f64) {
    let eirp = t.tx_power_dbm + t.tx_gain_dbi - t.tx_loss_db;
    let total_loss = fspl + diff_loss + misc_loss + t.rx_loss_db;
    let rx_pwr = eirp + t.rx_gain_dbi - total_loss;
    let margin = rx_pwr - t.rx_sensitivity_dbm - t.fade_margin_db;
    (eirp, rx_pwr, margin)
}

/// Computes complete RF link budget and Fresnel clearance summary.
pub fn calculate_rf_link_budget(
    t: &RfTransceiver,
    tx_alt_m: f64,
    rx_alt_m: f64,
    total_dist_m: f64,
    obs: &[ObstaclePoint],
    cfg: &FresnelCalculatorConfig,
) -> LinkBudgetResult {
    let d = total_dist_m.max(1.0);
    let prof = analyze_fresnel_profile(tx_alt_m, rx_alt_m, d, obs, cfg);
    let (min_r, worst_pt, diff) = find_worst_obstruction(&prof, d, cfg.frequency_hz);
    let fspl = compute_fspl(d, cfg.frequency_hz);
    let (eirp, rx_pwr, margin) = compute_rx_power_and_margin(t, fspl, diff, cfg.misc_loss_db);
    LinkBudgetResult {
        frequency_hz: cfg.frequency_hz,
        total_distance_m: d,
        eirp_dbm: eirp,
        fspl_db: fspl,
        diffraction_loss_db: diff,
        misc_loss_db: cfg.misc_loss_db,
        rx_power_dbm: rx_pwr,
        link_margin_db: margin,
        worst_clearance_ratio: min_r,
        worst_point: worst_pt,
        quality: classify_link_quality(margin),
    }
}
