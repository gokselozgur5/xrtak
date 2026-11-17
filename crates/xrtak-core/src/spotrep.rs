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

impl SpotReport {
    /// Serializes SPOTREP incident report to standard Cursor-on-Target XML string.
    pub fn to_cot_xml(&self) -> String {
        let (uid, typ, time) = (
            escape_xml(&self.cot_uid),
            escape_xml(&self.cot_type),
            escape_xml(&self.time_iso),
        );
        let detail = format_spotrep_detail_xml(self);
        format!(
            "<event version=\"2.0\" uid=\"{uid}\" type=\"{typ}\" \
time=\"{time}\" start=\"{time}\" stale=\"{time}\" how=\"m-g\">\
<point lat=\"{:.6}\" lon=\"{:.6}\" \
hae=\"{:.1}\" ce=\"9999999\" le=\"9999999\"/>\
{detail}</event>",
            self.lat, self.lon, self.hae
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpotrepTag {
    None,
    Size,
    Activity,
    Location,
    Unit,
    Time,
    Equipment,
    Assessment,
    Remarks,
}

fn parse_f64(bytes: &[u8]) -> Option<f64> {
    std::str::from_utf8(bytes).ok()?.trim().parse().ok()
}

fn match_tag(name: &[u8]) -> SpotrepTag {
    match name {
        b"size" => SpotrepTag::Size,
        b"activity" => SpotrepTag::Activity,
        b"location" => SpotrepTag::Location,
        b"unit" => SpotrepTag::Unit,
        b"time" => SpotrepTag::Time,
        b"equipment" => SpotrepTag::Equipment,
        b"assessment" => SpotrepTag::Assessment,
        b"remarks" => SpotrepTag::Remarks,
        _ => SpotrepTag::None,
    }
}

fn parse_spotrep_attributes(e: &quick_xml::events::BytesStart, r: &mut SpotReport) {
    for a in e.attributes().flatten() {
        let val = unescape_xml(&String::from_utf8_lossy(&a.value));
        match a.key.as_ref() {
            b"size" => r.size = val,
            b"activity" => r.activity = val,
            b"location" => r.location = val,
            b"unit" => r.unit = val,
            b"time" => r.time_observed = val,
            b"equipment" => r.equipment = val,
            b"assessment" => r.assessment = val,
            b"remarks" => r.remarks = val,
            b"observer" => r.observer_callsign = val,
            _ => {}
        }
    }
}

fn apply_text_to_field(tag: SpotrepTag, text: &str, r: &mut SpotReport) {
    let unescaped = unescape_xml(text);
    match tag {
        SpotrepTag::Size => r.size = unescaped,
        SpotrepTag::Activity => r.activity = unescaped,
        SpotrepTag::Location => r.location = unescaped,
        SpotrepTag::Unit => r.unit = unescaped,
        SpotrepTag::Time => r.time_observed = unescaped,
        SpotrepTag::Equipment => r.equipment = unescaped,
        SpotrepTag::Assessment => r.assessment = unescaped,
        SpotrepTag::Remarks => r.remarks = unescaped,
        SpotrepTag::None => {}
    }
}

fn parse_event_point(e: &quick_xml::events::BytesStart, r: &mut SpotReport, name: &[u8]) {
    for a in e.attributes().flatten() {
        if name == b"event" {
            if a.key.as_ref() == b"uid" { r.cot_uid = String::from_utf8_lossy(&a.value).into_owned(); }
            if a.key.as_ref() == b"type" { r.cot_type = String::from_utf8_lossy(&a.value).into_owned(); }
            if a.key.as_ref() == b"time" { r.time_iso = String::from_utf8_lossy(&a.value).into_owned(); }
        } else if name == b"point" {
            match a.key.as_ref() {
                b"lat" => r.lat = parse_f64(&a.value).unwrap_or(0.0),
                b"lon" => r.lon = parse_f64(&a.value).unwrap_or(0.0),
                b"hae" => r.hae = parse_f64(&a.value).unwrap_or(0.0),
                _ => {}
            }
        } else if name == b"contact" && a.key.as_ref() == b"callsign" {
            r.observer_callsign = unescape_xml(&String::from_utf8_lossy(&a.value));
        }
    }
}
