//! RF Link Budget, 1st Fresnel Zone & ITU-R Knife-Edge Diffraction Engine.
//!
//! Complies with REQ-RF-01..02 and ICD-19 for tactical datalink margin analysis.

use serde::{Deserialize, Serialize};

/// Standard tactical RF frequency bands and radio link profiles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RfBand {
    /// 900 MHz LoRa / Telemetry / ISR Command link.
    UHF900,
    /// 2.4 GHz MAVLink / C2 / Wi-Fi mesh.
    ISM2400,
    /// 5.8 GHz High-Definition Video / Silvus SC4200 / MPU5 MIMO.
    SHF5800,
}

impl RfBand {
    /// Nominal carrier frequency in Hertz (Hz).
    pub fn frequency_hz(&self) -> f64 {
        match self {
            RfBand::UHF900 => 900_000_000.0,
            RfBand::ISM2400 => 2_400_000_000.0,
            RfBand::SHF5800 => 5_800_000_000.0,
        }
    }

    /// Typical transmitter output power in dBm (1 Watt = +30 dBm).
    pub fn default_tx_power_dbm(&self) -> f64 {
        match self {
            RfBand::UHF900 => 30.0,   // 1W
            RfBand::ISM2400 => 27.0,  // 500mW
            RfBand::SHF5800 => 33.0,  // 2W MIMO
        }
    }

    /// Default receiver sensitivity threshold in dBm.
    pub fn default_rx_sensitivity_dbm(&self) -> f64 {
        match self {
            RfBand::UHF900 => -110.0,
            RfBand::ISM2400 => -95.0,
            RfBand::SHF5800 => -88.0,
        }
    }
}

/// Evaluated RF Link telemetry and propagation budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfLinkResult {
    pub frequency_hz: f64,
    pub wavelength_m: f64,
    pub distance_m: f64,
    pub fspl_db: f64,
    pub max_fresnel_radius_m: f64,
    pub worst_clearance_m: f64,
    pub diffraction_loss_db: f64,
    pub received_power_dbm: f64,
    pub link_margin_db: f64,
    pub status: String,
}

/// Computes Free-Space Path Loss (FSPL) in decibels.
pub fn compute_fspl_db(distance_m: f64, frequency_hz: f64) -> f64 {
    let d = distance_m.max(1.0);
    let c = 299_792_458.0;
    let lambda = c / frequency_hz;
    20.0 * (4.0 * std::f64::consts::PI * d / lambda).log10()
}

/// Computes 1st Fresnel zone radius F_1 (meters) at distances d1, d2.
pub fn compute_fresnel_radius_m(d1_m: f64, d2_m: f64, frequency_hz: f64) -> f64 {
    let total_d = d1_m + d2_m;
    if total_d <= 1e-3 {
        return 0.0;
    }
    let c = 299_792_458.0;
    let lambda = c / frequency_hz;
    ((lambda * d1_m * d2_m) / total_d).sqrt()
}

/// Computes Knife-Edge Diffraction Loss J(v) in dB using ITU-R P.526-15 approximation.
///
/// v is the dimensionless diffraction parameter.
/// If v <= -0.7, obstacle is outside 0.6 Fresnel clearance (0 dB loss).
pub fn compute_knife_edge_diffraction_db(v: f64) -> f64 {
    if v <= -0.7 {
        0.0
    } else {
        let term = ((v - 0.1).powi(2) + 1.0).sqrt() + (v - 0.1);
        if term <= 0.0 {
            0.0
        } else {
            6.9 + 20.0 * term.log10()
        }
    }
}

/// Computes dimensionless Fresnel-Kirchhoff diffraction parameter v.
///
/// obstacle_penetration_m is obstacle elevation minus LOS ray height at obstacle distance.
pub fn compute_diffraction_v(obstacle_penetration_m: f64, d1_m: f64, d2_m: f64, frequency_hz: f64) -> f64 {
    let total_d = d1_m + d2_m;
    if total_d <= 1e-3 || d1_m <= 1e-3 || d2_m <= 1e-3 {
        return -1.0;
    }
    let c = 299_792_458.0;
    let lambda = c / frequency_hz;
    let scale = (2.0 * total_d / (lambda * d1_m * d2_m)).sqrt();
    obstacle_penetration_m * scale
}

/// Samples terrain along ray and computes distance, worst clearance, diffraction v, and FSPL.
pub fn evaluate_rf_propagation(
    tx: (f64, f64, f64),
    rx: (f64, f64, f64),
    freq: f64,
    steps: usize,
    provider: &dyn crate::geo::ElevationProvider,
) -> (f64, f64, f64, f64) {
    let dx = rx.0 - tx.0;
    let dy = rx.1 - tx.1;
    let dz = rx.2 - tx.2;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(1.0);
    let n = steps.max(16);
    let mut worst_v = -10.0f64;
    let mut min_clear = f64::MAX;

    for i in 1..n {
        let t = i as f64 / n as f64;
        let ray_alt = tx.2 + t * dz;
        let terrain_alt = provider.sample_relief_m(tx.0 + t * dx, tx.1 + t * dy);
        let clear = ray_alt - terrain_alt;
        if clear < min_clear {
            min_clear = clear;
        }
        let v = compute_diffraction_v(terrain_alt - ray_alt, t * dist, (1.0 - t) * dist, freq);
        if v > worst_v {
            worst_v = v;
        }
    }
    let fspl = compute_fspl_db(dist, freq);
    (dist, min_clear, worst_v, fspl)
}
