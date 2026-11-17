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

