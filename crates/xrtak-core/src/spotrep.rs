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
