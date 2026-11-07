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
