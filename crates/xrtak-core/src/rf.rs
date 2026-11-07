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
