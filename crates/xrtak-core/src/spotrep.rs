//! SPOTREP / SALUTE tactical incident report CoT generator and parser.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during SPOTREP serialization and parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpotReportError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingSpotrepTag,
}

impl fmt::Display for SpotReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingSpotrepTag => write!(f, "Missing spotrep tag"),
        }
    }
}

impl std::error::Error for SpotReportError {}

/// Tactical Spot Report (SPOTREP / SALUTE format) packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotReport {
    pub cot_uid: String,
    pub observer_callsign: String,
    pub cot_type: String,
    pub size: String,
    pub activity: String,
    pub location: String,
    pub unit: String,
    pub time_observed: String,
    pub equipment: String,
    pub assessment: String,
    pub remarks: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub time_iso: String,
}

impl SpotReport {
    pub fn new(cot_uid: impl Into<String>, observer: impl Into<String>) -> Self {
        Self {
            cot_uid: cot_uid.into(),
            observer_callsign: observer.into(),
            cot_type: "b-r-f-h-c".to_string(),
            size: String::new(),
            activity: String::new(),
            location: String::new(),
            unit: String::new(),
            time_observed: String::new(),
            equipment: String::new(),
            assessment: String::new(),
            remarks: String::new(),
            lat: 0.0,
            lon: 0.0,
            hae: 0.0,
            time_iso: "1970-01-01T00:00:00Z".to_string(),
        }
    }
}
