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

impl CasNineLine {
    pub fn with_coordinates(mut self, lat: f64, lon: f64, hae: f64) -> Self {
        self.lat = lat;
        self.lon = lon;
        self.hae = hae;
        self
    }

    pub fn with_time(mut self, time_iso: impl Into<String>) -> Self {
        self.time_iso = time_iso.into();
        self
    }

    pub fn with_remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = remarks.into();
        self
    }
}

/// Formats standard voice readback text for CAS 9-Line briefing.
pub fn format_cas_nine_line_text(b: &CasNineLine) -> String {
    format!(
        "--- 9-LINE CAS BRIEF ---\n\
         1. IP/BP: {}\n\
         2. Heading: {}\n\
         3. Distance: {}\n\
         4. Elevation: {}\n\
         5. Description: {}\n\
         6. Location: {}\n\
         7. Mark: {}\n\
         8. Friendlies: {}\n\
         9. Egress: {}\n\
         Remarks: {}",
        b.line1_ip_bp,
        b.line2_heading,
        b.line3_distance,
        b.line4_elevation,
        b.line5_description,
        b.line6_location,
        b.line7_mark_type,
        b.line8_friendlies,
        b.line9_egress,
        b.remarks
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_xml(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn format_cas_detail_xml(b: &CasNineLine) -> String {
    let (cs, rem) = (escape_xml(&b.jtac_callsign), escape_xml(&b.remarks));
    let (l1, l2, l3) = (escape_xml(&b.line1_ip_bp), escape_xml(&b.line2_heading), escape_xml(&b.line3_distance));
    let (l4, l5, l6) = (escape_xml(&b.line4_elevation), escape_xml(&b.line5_description), escape_xml(&b.line6_location));
    let (l7, l8, l9) = (escape_xml(&b.line7_mark_type), escape_xml(&b.line8_friendlies), escape_xml(&b.line9_egress));
    format!(
        "<detail><contact callsign=\"{cs}\"/>\
<cas9line line1=\"{l1}\" line2=\"{l2}\" line3=\"{l3}\" line4=\"{l4}\" line5=\"{l5}\" line6=\"{l6}\" line7=\"{l7}\" line8=\"{l8}\" line9=\"{l9}\" remarks=\"{rem}\">\
<line1>{l1}</line1><line2>{l2}</line2><line3>{l3}</line3><line4>{l4}</line4><line5>{l5}</line5>\
<line6>{l6}</line6><line7>{l7}</line7><line8>{l8}</line8><line9>{l9}</line9><remarks>{rem}</remarks>\
</cas9line><remarks>{rem}</remarks></detail>"
    )
}
