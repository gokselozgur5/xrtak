// crates/xrtak-core/src/ew.rs
//! Electronic Warfare (EW) Jammer-to-Signal & Burn-Through Analysis Engine (ICD-23, REQ-EW-01..02).

use serde::{Deserialize, Serialize};

/// Target link / EW frequency band.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EwTargetBand {
    /// GPS L1 C/A (1575.42 MHz)
    GnssL1,
    /// Tactical C2 / MAVLink (915.0 MHz)
    TacticalC2,
    /// High-throughput video downlink (2400.0 MHz)
    VideoDownlink,
    /// X-band radar (9500.0 MHz)
    RadarXBand,
}

impl EwTargetBand {
    /// Carrier center frequency in MHz.
    pub fn frequency_mhz(&self) -> f64 {
        match self {
            Self::GnssL1 => 1575.42,
            Self::TacticalC2 => 915.0,
            Self::VideoDownlink => 2400.0,
            Self::RadarXBand => 9500.0,
        }
    }
}

/// Jammer parameters and operational geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JammerProfile {
    pub callsign: String,
    pub transmitter_power_w: f64,
    pub antenna_gain_dbi: f64,
    pub bandwidth_mhz: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
}

/// Target receiver / transceiver parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLinkProfile {
    pub callsign: String,
    pub transmitter_power_w: f64,
    pub tx_antenna_gain_dbi: f64,
    pub rx_antenna_gain_dbi: f64,
    pub signal_bandwidth_mhz: f64,
    pub receiver_sensitivity_dbm: f64,
    pub tx_east_m: f64,
    pub tx_north_m: f64,
    pub tx_up_m: f64,
    pub rx_east_m: f64,
    pub rx_north_m: f64,
    pub rx_up_m: f64,
}
