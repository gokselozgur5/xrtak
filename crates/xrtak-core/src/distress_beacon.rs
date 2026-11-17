//! CoT emergency beacon and 911 distress alert generator.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors during emergency beacon parsing and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistressBeaconError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingEmergencyTag,
    InvalidCoordinate(String),
}

impl fmt::Display for DistressBeaconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingEmergencyTag => write!(f, "Missing emergency tag"),
            Self::InvalidCoordinate(e) => write!(f, "Invalid coordinate: {e}"),
        }
    }
}

impl std::error::Error for DistressBeaconError {}

/// Tactical emergency distress alert category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyType {
    Alert911,
    RingTheBell,
    InContact,
    Ambush,
    Sniper,
    Ied,
    Medical,
    Custom(String),
}

impl EmergencyType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Alert911 => "911 Alert",
            Self::RingTheBell => "Ring the Bell",
            Self::InContact => "In Contact",
            Self::Ambush => "Ambush",
            Self::Sniper => "Sniper",
            Self::Ied => "IED",
            Self::Medical => "Medical",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl EmergencyType {
    pub fn from_str_type(s: &str) -> Self {
        match s {
            "911 Alert" => Self::Alert911,
            "Ring the Bell" => Self::RingTheBell,
            "In Contact" => Self::InContact,
            "Ambush" => Self::Ambush,
            "Sniper" => Self::Sniper,
            "IED" => Self::Ied,
            "Medical" => Self::Medical,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Operating state of an emergency distress beacon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyStatus {
    Active,
    Cancelled,
}

impl EmergencyStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn cancel_attr(&self) -> &'static str {
        match self {
            Self::Active => "false",
            Self::Cancelled => "true",
        }
    }
}

/// Tactical urgency level for distress situations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencySeverity {
    Critical,
    High,
    Moderate,
    Low,
}

impl EmergencySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Moderate => "MODERATE",
            Self::Low => "LOW",
        }
    }

    pub fn from_str_severity(s: &str) -> Self {
        match s {
            "CRITICAL" => Self::Critical,
            "HIGH" => Self::High,
            "LOW" => Self::Low,
            _ => Self::Moderate,
        }
    }
}

/// Specific tactical emergency condition indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SosFlags {
    pub officer_down: bool,
    pub under_fire: bool,
    pub medical_critical: bool,
    pub trapped: bool,
    pub silent_alarm: bool,
    pub radio_failure: bool,
}

impl SosFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_indicators(&self) -> Vec<&'static str> {
        let mut flags = Vec::new();
        if self.officer_down { flags.push("OFFICER_DOWN"); }
        if self.under_fire { flags.push("UNDER_FIRE"); }
        if self.medical_critical { flags.push("MED_CRITICAL"); }
        if self.trapped { flags.push("TRAPPED"); }
        if self.silent_alarm { flags.push("SILENT_ALARM"); }
        if self.radio_failure { flags.push("RADIO_FAIL"); }
        flags
    }
}

/// Cursor-on-Target Emergency Beacon / 911 distress alert record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistressBeacon {
    pub beacon_uid: String,
    pub callsign: String,
    pub alert_type: EmergencyType,
    pub status: EmergencyStatus,
    pub severity: EmergencySeverity,
    pub sos_flags: SosFlags,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub ce: f64,
    pub le: f64,
    pub time_iso: String,
    pub stale_iso: String,
    pub description: String,
    pub parent_uid: Option<String>,
}

impl DistressBeacon {
    pub fn new(
        beacon_uid: impl Into<String>,
        callsign: impl Into<String>,
        alert_type: EmergencyType,
        lat: f64,
        lon: f64,
    ) -> Self {
        Self {
            beacon_uid: beacon_uid.into(),
            callsign: callsign.into(),
            alert_type,
            status: EmergencyStatus::Active,
            severity: EmergencySeverity::Critical,
            sos_flags: SosFlags::default(),
            lat,
            lon,
            hae: 0.0,
            ce: 10.0,
            le: 10.0,
            time_iso: "2026-01-01T00:00:00Z".to_string(),
            stale_iso: "2026-01-01T00:10:00Z".to_string(),
            description: String::new(),
            parent_uid: None,
        }
    }
}

impl DistressBeacon {
    pub fn with_elevation(mut self, hae: f64, ce: f64, le: f64) -> Self {
        self.hae = hae;
        self.ce = ce;
        self.le = le;
        self
    }

    pub fn with_severity(mut self, severity: EmergencySeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_sos_flags(mut self, sos_flags: SosFlags) -> Self {
        self.sos_flags = sos_flags;
        self
    }
}

impl DistressBeacon {
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_parent_uid(mut self, parent_uid: impl Into<String>) -> Self {
        self.parent_uid = Some(parent_uid.into());
        self
    }

    pub fn with_timestamps(
        mut self,
        time_iso: impl Into<String>,
        stale_iso: impl Into<String>,
    ) -> Self {
        self.time_iso = time_iso.into();
        self.stale_iso = stale_iso.into();
        self
    }
}

impl DistressBeacon {
    pub fn cancel(&mut self, timestamp_iso: impl Into<String>) {
        self.status = EmergencyStatus::Cancelled;
        self.time_iso = timestamp_iso.into();
    }

    pub fn activate(&mut self, timestamp_iso: impl Into<String>) {
        self.status = EmergencyStatus::Active;
        self.time_iso = timestamp_iso.into();
    }

    pub fn cot_type(&self) -> &'static str {
        match self.status {
            EmergencyStatus::Cancelled => "b-a-o-can",
            EmergencyStatus::Active => match self.alert_type {
                EmergencyType::InContact => "b-a-o-tbl",
                _ => "b-a-o-pan",
            },
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace(''', "&apos;")
}

impl DistressBeacon {
    pub fn format_distress_voice_readback(&self) -> String {
        let status_str = match self.status {
            EmergencyStatus::Active => "MAYDAY MAYDAY MAYDAY",
            EmergencyStatus::Cancelled => "EMERGENCY CANCELLED",
        };
        let flags = self.sos_flags.active_indicators();
        let flags_str = if flags.is_empty() {
            "NONE".to_string()
        } else {
            flags.join(", ")
        };
        let desc = if self.description.is_empty() { "NONE" } else { &self.description };
        format!(
            "{status_str}\n\
             BEACON: {}\n\
             CALLSIGN: {}\n\
             TYPE: {}\n\
             SEVERITY: {}\n\
             FLAGS: {}\n\
             POSITION: {:.6}, {:.6} (HAE: {:.1}m)\n\
             REMARKS: {}",
            self.beacon_uid, self.callsign, self.alert_type.as_str(),
            self.severity.as_str(), flags_str, self.lat, self.lon, self.hae, desc
        )
    }
}

impl DistressBeacon {
    pub fn to_cot_xml(&self) -> String {
        let cot_type = self.cot_type();
        let cancel_str = self.status.cancel_attr();
        let alert_type_str = escape_xml(self.alert_type.as_str());
        let callsign_esc = escape_xml(&self.callsign);
        let uid_esc = escape_xml(&self.beacon_uid);
        let desc_esc = escape_xml(&self.description);
        let mut xml = format!(
            "<event version=\"2.0\" uid=\"{uid_esc}\" type=\"{cot_type}\"              how=\"m-g\" time=\"{}\" start=\"{}\" stale=\"{}\">             <point lat=\"{:.7}\" lon=\"{:.7}\" hae=\"{:.2}\" ce=\"{:.1}\" le=\"{:.1}\"/>             <detail>             <emergency cancel=\"{cancel_str}\" type=\"{alert_type_str}\">{callsign_esc}</emergency>             <contact callsign=\"{callsign_esc}\"/>",
            self.time_iso, self.time_iso, self.stale_iso,
            self.lat, self.lon, self.hae, self.ce, self.le
        );
        if !self.description.is_empty() {
            xml.push_str(&format!("<remarks>{desc_esc}</remarks>"));
        }
        if let Some(ref parent) = self.parent_uid {
            let parent_esc = escape_xml(parent);
            xml.push_str(&format!(
                "<link relation=\"p-p\" type=\"a-f-G-U-C\" uid=\"{parent_esc}\"/>"
            ));
        }
        xml.push_str("</detail></event>");
        xml
    }
}

fn parse_f64(val: &[u8]) -> Option<f64> {
    std::str::from_utf8(val).ok()?.parse::<f64>().ok()
}

fn parse_str_attr(val: &[u8]) -> String {
    std::str::from_utf8(val).unwrap_or_default().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BeaconXmlTag {
    None,
    Event,
    Point,
    Emergency,
    Contact,
    Remarks,
    Link,
}

impl DistressBeacon {
    pub fn from_cot_xml(xml: &str) -> Result<Self, DistressBeaconError> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut uid = None;
        let mut cot_type = None;
        let mut time_iso = None;
        let mut stale_iso = None;
        let mut point = None;
        let mut callsign = None;
        let mut emergency_type = None;
        let mut is_cancelled = false;
        let mut emergency_text = String::new();
        let mut remarks = String::new();
        let mut parent_uid = None;
        let mut current_tag = BeaconXmlTag::None;
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"event" => {
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"uid" => uid = Some(parse_str_attr(&a.value)),
                                b"type" => cot_type = Some(parse_str_attr(&a.value)),
                                b"time" => time_iso = Some(parse_str_attr(&a.value)),
                                b"stale" => stale_iso = Some(parse_str_attr(&a.value)),
                                _ => {}
                            }
                        }
                        current_tag = BeaconXmlTag::Event;
                    }
                    b"point" => {
                        let mut lat = None;
                        let mut lon = None;
                        let mut hae = 0.0;
                        let mut ce = 10.0;
                        let mut le = 10.0;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"lat" => lat = parse_f64(&a.value),
                                b"lon" => lon = parse_f64(&a.value),
                                b"hae" => hae = parse_f64(&a.value).unwrap_or(0.0),
                                b"ce" => ce = parse_f64(&a.value).unwrap_or(10.0),
                                b"le" => le = parse_f64(&a.value).unwrap_or(10.0),
                                _ => {}
                            }
                        }
                        if let (Some(la), Some(lo)) = (lat, lon) {
                            point = Some((la, lo, hae, ce, le));
                        }
                        current_tag = BeaconXmlTag::Point;
                    }
                    b"emergency" => {
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"cancel" => {
                                    if parse_str_attr(&a.value) == "true" {
                                        is_cancelled = true;
                                    }
                                }
                                b"type" => {
                                    let s = parse_str_attr(&a.value);
                                    emergency_type = Some(EmergencyType::from_str_type(&s));
                                }
                                _ => {}
                            }
                        }
                        current_tag = BeaconXmlTag::Emergency;
                    }
                    b"contact" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"callsign" {
                                callsign = Some(parse_str_attr(&a.value));
                            }
                        }
                        current_tag = BeaconXmlTag::Contact;
                    }
                    b"remarks" => current_tag = BeaconXmlTag::Remarks,
                    b"link" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"uid" {
                                parent_uid = Some(parse_str_attr(&a.value));
                            }
                        }
                        current_tag = BeaconXmlTag::Link;
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"point" => {
                        let mut lat = None;
                        let mut lon = None;
                        let mut hae = 0.0;
                        let mut ce = 10.0;
                        let mut le = 10.0;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"lat" => lat = parse_f64(&a.value),
                                b"lon" => lon = parse_f64(&a.value),
                                b"hae" => hae = parse_f64(&a.value).unwrap_or(0.0),
                                b"ce" => ce = parse_f64(&a.value).unwrap_or(10.0),
                                b"le" => le = parse_f64(&a.value).unwrap_or(10.0),
                                _ => {}
                            }
                        }
                        if let (Some(la), Some(lo)) = (lat, lon) {
                            point = Some((la, lo, hae, ce, le));
                        }
                    }
                    b"emergency" => {
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"cancel" => {
                                    if parse_str_attr(&a.value) == "true" {
                                        is_cancelled = true;
                                    }
                                }
                                b"type" => {
                                    let s = parse_str_attr(&a.value);
                                    emergency_type = Some(EmergencyType::from_str_type(&s));
                                }
                                _ => {}
                            }
                        }
                    }
                    b"contact" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"callsign" {
                                callsign = Some(parse_str_attr(&a.value));
                            }
                        }
                    }
                    b"link" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"uid" {
                                parent_uid = Some(parse_str_attr(&a.value));
                            }
                        }
                    }
                    _ => {}
                },
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().into_owned();
                    match current_tag {
                        BeaconXmlTag::Emergency => emergency_text.push_str(&text),
                        BeaconXmlTag::Remarks => remarks.push_str(&text),
                        _ => {}
                    }
                }
                Ok(Event::End(_)) => current_tag = BeaconXmlTag::None,
                Ok(Event::Eof) => break,
                Err(err) => return Err(DistressBeaconError::InvalidXml(err.to_string())),
                _ => {}
            }
            buf.clear();
        }
        let beacon_uid = uid.ok_or(DistressBeaconError::MissingEventTag)?;
        let (lat, lon, hae, ce, le) = point.ok_or(DistressBeaconError::MissingPointTag)?;
        let emg_type = emergency_type.ok_or(DistressBeaconError::MissingEmergencyTag)?;
        let cs = callsign
            .or_else(|| {
                let trimmed = emergency_text.trim();
                if !trimmed.is_empty() { Some(trimmed.to_string()) } else { None }
            })
            .unwrap_or_else(|| beacon_uid.clone());
        let status = if is_cancelled || cot_type.as_deref() == Some("b-a-o-can") {
            EmergencyStatus::Cancelled
        } else {
            EmergencyStatus::Active
        };
        Ok(Self {
            beacon_uid,
            callsign: cs,
            alert_type: emg_type,
            status,
            severity: EmergencySeverity::Critical,
            sos_flags: SosFlags::default(),
            lat,
            lon,
            hae,
            ce,
            le,
            time_iso: time_iso.unwrap_or_default(),
            stale_iso: stale_iso.unwrap_or_default(),
            description: remarks,
            parent_uid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_construction() {
        let b = DistressBeacon::new(
            "BEACON-01",
            "RESCUE-1",
            EmergencyType::Alert911,
            34.0522,
            -118.2437,
        );
        assert_eq!(b.beacon_uid, "BEACON-01");
        assert_eq!(b.callsign, "RESCUE-1");
        assert_eq!(b.alert_type, EmergencyType::Alert911);
        assert_eq!(b.status, EmergencyStatus::Active);
        assert_eq!(b.cot_type(), "b-a-o-pan");
    }
    #[test]
    fn test_beacon_state_transitions() {
        let mut b = DistressBeacon::new(
            "BEACON-02",
            "ALPHA-2",
            EmergencyType::InContact,
            34.0,
            -118.0,
        );
        assert_eq!(b.cot_type(), "b-a-o-tbl");
        b.cancel("2026-01-01T01:00:00Z");
        assert_eq!(b.status, EmergencyStatus::Cancelled);
        assert_eq!(b.cot_type(), "b-a-o-can");
        b.activate("2026-01-01T01:05:00Z");
        assert_eq!(b.status, EmergencyStatus::Active);
    }
