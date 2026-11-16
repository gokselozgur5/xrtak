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
            Self::EmptyChannelList => write!(f, "Channel allocation list is empty"),
            Self::ChannelNotFound(id) => write!(f, "Channel id {} not found", id),
        }
    }
}

impl std::error::Error for CommsChannelError {}

/// Tactical radio communications frequency & channel allocation packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommsChannelPacket {
    pub uid: String,
    pub cot_type: String,
    pub callsign: String,
    pub timestamp_iso: String,
    pub stale_iso: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub channels: Vec<RadioChannel>,
    pub primary_channel_id: Option<u32>,
    pub emergency_channel_id: Option<u32>,
}

impl CommsChannelPacket {
    pub fn new(
        uid: impl Into<String>,
        callsign: impl Into<String>,
        timestamp_iso: impl Into<String>,
        stale_iso: impl Into<String>,
    ) -> Self {
        Self {
            uid: uid.into(),
            cot_type: "a-f-G-U-C".to_string(),
            callsign: callsign.into(),
            timestamp_iso: timestamp_iso.into(),
            stale_iso: stale_iso.into(),
            lat: 0.0,
            lon: 0.0,
            hae: 0.0,
            channels: Vec::new(),
            primary_channel_id: None,
            emergency_channel_id: None,
        }
    }

    pub fn with_coordinates(mut self, lat: f64, lon: f64, hae: f64) -> Self {
        self.lat = lat;
        self.lon = lon;
        self.hae = hae;
        self
    }

    pub fn with_cot_type(mut self, cot_type: impl Into<String>) -> Self {
        self.cot_type = cot_type.into();
        self
    }

    pub fn add_channel(mut self, channel: RadioChannel) -> Self {
        self.channels.push(channel);
        self
    }

    pub fn with_primary_channel(mut self, channel_id: u32) -> Self {
        self.primary_channel_id = Some(channel_id);
        self
    }

    pub fn with_emergency_channel(mut self, channel_id: u32) -> Self {
        self.emergency_channel_id = Some(channel_id);
        self
    }

    pub fn primary_channel(&self) -> Option<&RadioChannel> {
        self.primary_channel_id
            .and_then(|id| self.channels.iter().find(|c| c.id == id))
    }

    pub fn emergency_channel(&self) -> Option<&RadioChannel> {
        self.emergency_channel_id
            .and_then(|id| self.channels.iter().find(|c| c.id == id))
    }

    pub fn find_channel(&self, id: u32) -> Option<&RadioChannel> {
        self.channels.iter().find(|c| c.id == id)
    }

    /// Serializes to Cursor-on-Target XML with `<radioChannels>` detail.
    pub fn to_cot_xml(&self) -> String {
        let uid = escape_xml(&self.uid);
        let ctype = escape_xml(&self.cot_type);
        let callsign = escape_xml(&self.callsign);
        let t = &self.timestamp_iso;
        let s = &self.stale_iso;

        let mut out = String::new();
        out.push_str(&format!(
            "<event version=\"2.0\" uid=\"{uid}\" type=\"{ctype}\" how=\"m-g\" time=\"{t}\" start=\"{t}\" stale=\"{s}\">"
        ));
        out.push_str(&format!(
            "<point lat=\"{:.6}\" lon=\"{:.6}\" hae=\"{:.1}\" ce=\"10.0\" le=\"5.0\"/>",
            self.lat, self.lon, self.hae
        ));
        out.push_str("<detail>");
        out.push_str(&format!("<contact callsign=\"{callsign}\"/>"));

        let mut radio_attr = String::new();
        if let Some(pri) = self.primary_channel_id {
            radio_attr.push_str(&format!(" primary=\"{pri}\""));
        }
        if let Some(emg) = self.emergency_channel_id {
            radio_attr.push_str(&format!(" emergency=\"{emg}\""));
        }

        out.push_str(&format!("<radioChannels{radio_attr}>"));
        for ch in &self.channels {
            let ch_name = escape_xml(&ch.name);
            let mode_str = ch.mode.as_str();
            let mut ch_xml = format!(
                "<channel id=\"{}\" name=\"{}\" frequency=\"{}\" bandwidth=\"{}\" mode=\"{}\"",
                ch.id, ch_name, ch.frequency_hz, ch.bandwidth_hz, mode_str
            );
            if let Some(pwr) = ch.tx_power_watts {
                ch_xml.push_str(&format!(" power=\"{:.1}\"", pwr));
            }
            if let Some(preset) = &ch.net_preset {
                ch_xml.push_str(&format!(" netPreset=\"{}\"", escape_xml(preset)));
            }
            if let Some(enc) = &ch.encryption {
                ch_xml.push_str(&format!(" encryption=\"{}\"", escape_xml(enc)));
            }
            ch_xml.push_str("/>");
            out.push_str(&ch_xml);
        }
        out.push_str("</radioChannels></detail></event>");
        out
    }

    /// Parses Cursor-on-Target XML into a CommsChannelPacket.
    pub fn from_cot_xml(xml: &str) -> Result<Self, CommsChannelError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut found_event = false;
        let mut found_radio_channels = false;
        let mut uid = String::new();
        let mut cot_type = "a-f-G-U-C".to_string();
        let mut callsign = String::new();
        let mut time_str = String::new();
        let mut stale_str = String::new();
        let (mut lat, mut lon, mut hae) = (0.0, 0.0, 0.0);
        let mut primary_id = None;
        let mut emergency_id = None;
        let mut channels = Vec::new();
