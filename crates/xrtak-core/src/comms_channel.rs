//! Tactical Radio Communications Frequency and Channel Allocation CoT packet builder.
//!
//! Complies with ATAK radio channel definitions and tactical net plans.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Tactical radio modulation and transmission mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadioMode {
    Am,
    Fm,
    NarrowbandFm,
    FrequencyHopping,
    Digital,
}

impl RadioMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Am => "AM",
            Self::Fm => "FM",
            Self::NarrowbandFm => "FMN",
            Self::FrequencyHopping => "FH",
            Self::Digital => "DIGITAL",
        }
    }

    pub fn from_mode_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "AM" => Self::Am,
            "FMN" | "NFM" | "NARROWBAND_FM" => Self::NarrowbandFm,
            "FH" | "FREQ_HOPPING" => Self::FrequencyHopping,
            "DIGITAL" | "DMR" | "P25" => Self::Digital,
            _ => Self::Fm,
        }
    }
}

/// Tactical radio channel allocation entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioChannel {
    pub id: u32,
    pub name: String,
    pub frequency_hz: u64,
    pub bandwidth_hz: u32,
    pub mode: RadioMode,
    pub tx_power_watts: Option<f64>,
    pub net_preset: Option<String>,
    pub encryption: Option<String>,
}

impl RadioChannel {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        frequency_hz: u64,
        bandwidth_hz: u32,
        mode: RadioMode,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            frequency_hz,
            bandwidth_hz,
            mode,
            tx_power_watts: None,
            net_preset: None,
            encryption: None,
        }
    }

    pub fn with_tx_power(mut self, power_watts: f64) -> Self {
        self.tx_power_watts = Some(power_watts);
        self
    }

    pub fn with_net_preset(mut self, preset: impl Into<String>) -> Self {
        self.net_preset = Some(preset.into());
        self
    }

    pub fn with_encryption(mut self, encryption: impl Into<String>) -> Self {
        self.encryption = Some(encryption.into());
        self
    }
}

/// Errors during CoT radio channel packet parsing or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommsChannelError {
    InvalidXml(String),
    MissingEventTag,
    MissingUid,
    MissingCallsign,
    MissingRadioChannelsTag,
    EmptyChannelList,
    ChannelNotFound(u32),
}

impl fmt::Display for CommsChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {}", e),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingUid => write!(f, "Missing event UID"),
            Self::MissingCallsign => write!(f, "Missing contact callsign"),
            Self::MissingRadioChannelsTag => write!(f, "Missing radioChannels tag"),
