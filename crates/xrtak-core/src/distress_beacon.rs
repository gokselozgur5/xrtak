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

