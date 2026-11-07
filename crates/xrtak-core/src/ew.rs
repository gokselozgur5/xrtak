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
