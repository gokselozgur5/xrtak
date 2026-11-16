//! CoT 9-Line Close Air Support (CAS) briefing XML generator and parser.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during CAS 9-Line parsing and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CasNineLineError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingCasTag,
}

impl fmt::Display for CasNineLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingCasTag => write!(f, "Missing cas9line tag"),
        }
    }
}

impl std::error::Error for CasNineLineError {}

/// CoT 9-Line Close Air Support (CAS) briefing packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CasNineLine {
    pub cot_uid: String,
    pub jtac_callsign: String,
    pub line1_ip_bp: String,
    pub line2_heading: String,
    pub line3_distance: String,
    pub line4_elevation: String,
    pub line5_description: String,
    pub line6_location: String,
    pub line7_mark_type: String,
    pub line8_friendlies: String,
    pub line9_egress: String,
    pub remarks: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub time_iso: String,
}

impl CasNineLine {
    pub fn new(cot_uid: impl Into<String>, jtac: impl Into<String>) -> Self {
        Self {
            cot_uid: cot_uid.into(),
            jtac_callsign: jtac.into(),
            line1_ip_bp: String::new(),
            line2_heading: String::new(),
            line3_distance: String::new(),
            line4_elevation: String::new(),
            line5_description: String::new(),
            line6_location: String::new(),
            line7_mark_type: String::new(),
            line8_friendlies: String::new(),
            line9_egress: String::new(),
            remarks: String::new(),
            lat: 0.0,
            lon: 0.0,
            hae: 0.0,
            time_iso: "1970-01-01T00:00:00Z".to_string(),
        }
    }
}
