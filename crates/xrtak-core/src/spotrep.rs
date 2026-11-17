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

impl SpotReport {
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

    pub fn with_assessment(mut self, assessment: impl Into<String>) -> Self {
        self.assessment = assessment.into();
        self
    }
}

impl SpotReport {
    pub fn with_salute(
        mut self,
        size: impl Into<String>,
        activity: impl Into<String>,
        location: impl Into<String>,
        unit: impl Into<String>,
        time_observed: impl Into<String>,
        equipment: impl Into<String>,
    ) -> Self {
        self.size = size.into();
        self.activity = activity.into();
        self.location = location.into();
        self.unit = unit.into();
        self.time_observed = time_observed.into();
        self.equipment = equipment.into();
        self
    }

    pub fn with_cot_type(mut self, cot_type: impl Into<String>) -> Self {
        self.cot_type = cot_type.into();
        self
    }
}

/// Formats standard voice or textual readback text for SPOTREP (SALUTE format).
pub fn format_salute_report_text(report: &SpotReport) -> String {
    format!(
        "--- SPOT REPORT (SALUTE) ---\n\
         1. S - SIZE: {}\n\
         2. A - ACTIVITY: {}\n\
         3. L - LOCATION: {}\n\
         4. U - UNIT: {}\n\
         5. T - TIME: {}\n\
         6. E - EQUIPMENT: {}\n\
         Assessment: {}\n\
         Remarks: {}",
        report.size,
        report.activity,
        report.location,
        report.unit,
        report.time_observed,
        report.equipment,
        report.assessment,
        report.remarks
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

fn format_spotrep_detail_xml(r: &SpotReport) -> String {
    let (cs, rem, ass) = (
        escape_xml(&r.observer_callsign),
        escape_xml(&r.remarks),
        escape_xml(&r.assessment),
    );
    let (sz, act, loc) = (
        escape_xml(&r.size),
        escape_xml(&r.activity),
        escape_xml(&r.location),
    );
    let (un, tm, eq) = (
        escape_xml(&r.unit),
        escape_xml(&r.time_observed),
        escape_xml(&r.equipment),
    );
    format!(
        "<detail><contact callsign=\"{cs}\"/><remarks>{rem}</remarks>\
<spotrep size=\"{sz}\" activity=\"{act}\" location=\"{loc}\" unit=\"{un}\" time=\"{tm}\" equipment=\"{eq}\" assessment=\"{ass}\">\
<size>{sz}</size><activity>{act}</activity><location>{loc}</location>\
<unit>{un}</unit><time>{tm}</time><equipment>{eq}</equipment><assessment>{ass}</assessment>\
</spotrep></detail>"
    )
}
