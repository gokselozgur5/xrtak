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
        let t = escape_xml(&self.timestamp_iso);
        let s = escape_xml(&self.stale_iso);

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
                ch_xml.push_str(&format!(" power=\"{}\"", pwr));
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

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"event" => {
                        found_event = true;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"uid" => {
                                    uid = a.unescape_value().map(|v| v.into_owned()).unwrap_or_else(
                                        |_| String::from_utf8_lossy(&a.value).into_owned(),
                                    )
                                }
                                b"type" => {
                                    cot_type = String::from_utf8_lossy(&a.value).into_owned()
                                }
                                b"time" => {
                                    time_str = String::from_utf8_lossy(&a.value).into_owned()
                                }
                                b"stale" => {
                                    stale_str = String::from_utf8_lossy(&a.value).into_owned()
                                }
                                _ => {}
                            }
                        }
                    }
                    b"point" => {
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"lat" => lat = parse_f64(&a.value).unwrap_or(0.0),
                                b"lon" => lon = parse_f64(&a.value).unwrap_or(0.0),
                                b"hae" => hae = parse_f64(&a.value).unwrap_or(0.0),
                                _ => {}
                            }
                        }
                    }
                    b"contact" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"callsign" {
                                callsign = String::from_utf8_lossy(&a.value).into_owned();
                            }
                        }
                    }
                    b"radioChannels" => {
                        found_radio_channels = true;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"primary" => primary_id = parse_u32(&a.value),
                                b"emergency" => emergency_id = parse_u32(&a.value),
                                _ => {}
                            }
                        }
                    }
                    b"channel" => {
                        let mut ch_id = 0u32;
                        let mut ch_name = String::new();
                        let mut ch_freq = 0u64;
                        let mut ch_bw = 25000u32;
                        let mut ch_mode = RadioMode::Fm;
                        let mut ch_pwr = None;
                        let mut ch_preset = None;
                        let mut ch_enc = None;

                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"id" => ch_id = parse_u32(&a.value).unwrap_or(0),
                                b"name" => {
                                    ch_name = unescape_xml(&String::from_utf8_lossy(&a.value))
                                }
                                b"frequency" => ch_freq = parse_u64(&a.value).unwrap_or(0),
                                b"bandwidth" => ch_bw = parse_u32(&a.value).unwrap_or(25000),
                                b"mode" => {
                                    let m_str = String::from_utf8_lossy(&a.value);
                                    ch_mode = RadioMode::from_mode_str(&m_str);
                                }
                                b"power" => ch_pwr = parse_f64(&a.value),
                                b"netPreset" => {
                                    ch_preset =
                                        Some(unescape_xml(&String::from_utf8_lossy(&a.value)))
                                }
                                b"encryption" => {
                                    ch_enc = Some(unescape_xml(&String::from_utf8_lossy(&a.value)))
                                }
                                _ => {}
                            }
                        }
                        channels.push(RadioChannel {
                            id: ch_id,
                            name: ch_name,
                            frequency_hz: ch_freq,
                            bandwidth_hz: ch_bw,
                            mode: ch_mode,
                            tx_power_watts: ch_pwr,
                            net_preset: ch_preset,
                            encryption: ch_enc,
                        });
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(CommsChannelError::InvalidXml(e.to_string())),
                _ => {}
            }
        }

        if !found_event {
            return Err(CommsChannelError::MissingEventTag);
        }
        if uid.is_empty() {
            return Err(CommsChannelError::MissingUid);
        }
        if !found_radio_channels {
            return Err(CommsChannelError::MissingRadioChannelsTag);
        }

        Ok(Self {
            uid,
            cot_type,
            callsign: unescape_xml(&callsign),
            timestamp_iso: time_str,
            stale_iso: stale_str,
            lat,
            lon,
            hae,
            channels,
            primary_channel_id: primary_id,
            emergency_channel_id: emergency_id,
        })
    }
}

fn parse_f64(bytes: &[u8]) -> Option<f64> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn parse_u32(bytes: &[u8]) -> Option<u32> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn parse_u64(bytes: &[u8]) -> Option<u64> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comms_channel_roundtrip() {
        let ch1 = RadioChannel::new(
            1,
            "TAC-1 Primary",
            225_500_000,
            25000,
            RadioMode::FrequencyHopping,
        )
        .with_tx_power(10.0)
        .with_net_preset("Alpha-Net")
        .with_encryption("AES-256");

        let ch2 = RadioChannel::new(2, "GUARD Air Distress", 121_500_000, 12500, RadioMode::Am)
            .with_tx_power(5.0);

        let packet = CommsChannelPacket::new(
            "COMMS-PLAN-001",
            "ALPHA-TOC",
            "2026-11-16T10:00:00Z",
            "2026-11-16T12:00:00Z",
        )
        .with_coordinates(34.0522, -118.2437, 120.0)
        .add_channel(ch1)
        .add_channel(ch2)
        .with_primary_channel(1)
        .with_emergency_channel(2);

        let xml = packet.to_cot_xml();
        assert!(xml.contains("radioChannels primary=\"1\" emergency=\"2\""));
        assert!(xml.contains("TAC-1 Primary"));

        let parsed = CommsChannelPacket::from_cot_xml(&xml).expect("Failed to parse roundtrip XML");
        assert_eq!(parsed.uid, "COMMS-PLAN-001");
        assert_eq!(parsed.callsign, "ALPHA-TOC");
        assert_eq!(parsed.channels.len(), 2);
        assert_eq!(parsed.primary_channel_id, Some(1));
        assert_eq!(parsed.emergency_channel_id, Some(2));

        let p_ch = parsed
            .primary_channel()
            .expect("primary channel should exist");
        assert_eq!(p_ch.name, "TAC-1 Primary");
        assert_eq!(p_ch.frequency_hz, 225_500_000);
        assert_eq!(p_ch.mode, RadioMode::FrequencyHopping);
        assert_eq!(p_ch.tx_power_watts, Some(10.0));
        assert_eq!(p_ch.net_preset.as_deref(), Some("Alpha-Net"));
        assert_eq!(p_ch.encryption.as_deref(), Some("AES-256"));

        let e_ch = parsed
            .emergency_channel()
            .expect("emergency channel should exist");
        assert_eq!(e_ch.name, "GUARD Air Distress");
        assert_eq!(e_ch.mode, RadioMode::Am);
    }

    #[test]
    fn test_channel_lookup_methods() {
        let ch = RadioChannel::new(10, "Convoy-Net", 156_800_000, 25000, RadioMode::Fm);
        let packet = CommsChannelPacket::new(
            "PLAN-2",
            "BRAVO",
            "2026-11-16T10:00:00Z",
            "2026-11-16T11:00:00Z",
        )
        .add_channel(ch);

        assert!(packet.find_channel(10).is_some());
        assert!(packet.find_channel(99).is_none());
        assert!(packet.primary_channel().is_none());
        assert!(packet.emergency_channel().is_none());
    }

    #[test]
    fn test_xml_attribute_escaping() {
        let ch = RadioChannel::new(
            1,
            "Ops & Intel <Air>",
            450_000_000,
            12500,
            RadioMode::NarrowbandFm,
        )
        .with_net_preset("Preset \"X\"")
        .with_encryption("Type-1 & Suite-B");

        let packet = CommsChannelPacket::new(
            "PLAN-ESC",
            "TOC <Leader>",
            "2026-11-16T10:00:00Z",
            "2026-11-16T11:00:00Z",
        )
        .add_channel(ch);

        let xml = packet.to_cot_xml();
        assert!(xml.contains("Ops &amp; Intel &lt;Air&gt;"));
        assert!(xml.contains("Preset &quot;X&quot;"));
        assert!(xml.contains("TOC &lt;Leader&gt;"));

        let parsed = CommsChannelPacket::from_cot_xml(&xml).expect("parsed escaped xml");
        assert_eq!(parsed.callsign, "TOC <Leader>");
        let ch_parsed = parsed.find_channel(1).unwrap();
        assert_eq!(ch_parsed.name, "Ops & Intel <Air>");
        assert_eq!(ch_parsed.net_preset.as_deref(), Some("Preset \"X\""));
        assert_eq!(ch_parsed.encryption.as_deref(), Some("Type-1 & Suite-B"));
    }

    #[test]
    fn test_error_handling() {
        let bad_xml = "<invalid_xml";
        assert!(matches!(
            CommsChannelPacket::from_cot_xml(bad_xml),
            Err(CommsChannelError::InvalidXml(_))
        ));

        let missing_event = "<someOtherRoot></someOtherRoot>";
        assert_eq!(
            CommsChannelPacket::from_cot_xml(missing_event),
            Err(CommsChannelError::MissingEventTag)
        );

        let missing_uid = "<event version=\"2.0\"><detail><radioChannels/></detail></event>";
        assert_eq!(
            CommsChannelPacket::from_cot_xml(missing_uid),
            Err(CommsChannelError::MissingUid)
        );

        let missing_channels = "<event version=\"2.0\" uid=\"U1\"><detail></detail></event>";
        assert_eq!(
            CommsChannelPacket::from_cot_xml(missing_channels),
            Err(CommsChannelError::MissingRadioChannelsTag)
        );
    }
}
