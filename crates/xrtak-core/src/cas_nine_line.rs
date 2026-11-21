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
    InvalidCoordinate(String),
}

impl fmt::Display for CasNineLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingCasTag => write!(f, "Missing cas9line tag"),
            Self::InvalidCoordinate(e) => write!(f, "Invalid coordinate: {e}"),
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
    let (l1, l2, l3) = (
        escape_xml(&b.line1_ip_bp),
        escape_xml(&b.line2_heading),
        escape_xml(&b.line3_distance),
    );
    let (l4, l5, l6) = (
        escape_xml(&b.line4_elevation),
        escape_xml(&b.line5_description),
        escape_xml(&b.line6_location),
    );
    let (l7, l8, l9) = (
        escape_xml(&b.line7_mark_type),
        escape_xml(&b.line8_friendlies),
        escape_xml(&b.line9_egress),
    );
    format!(
        "<detail><contact callsign=\"{cs}\"/>\
<cas9line line1=\"{l1}\" line2=\"{l2}\" line3=\"{l3}\" line4=\"{l4}\" line5=\"{l5}\" line6=\"{l6}\" line7=\"{l7}\" line8=\"{l8}\" line9=\"{l9}\" remarks=\"{rem}\">\
<line1>{l1}</line1><line2>{l2}</line2><line3>{l3}</line3><line4>{l4}</line4><line5>{l5}</line5>\
<line6>{l6}</line6><line7>{l7}</line7><line8>{l8}</line8><line9>{l9}</line9><remarks>{rem}</remarks>\
</cas9line><remarks>{rem}</remarks></detail>"
    )
}

impl CasNineLine {
    /// Serializes CAS 9-Line briefing into CoT XML string.
    pub fn to_cot_xml(&self) -> String {
        let uid = escape_xml(&self.cot_uid);
        let t = escape_xml(&self.time_iso);
        let detail = format_cas_detail_xml(self);
        format!(
            "<event version=\"2.0\" uid=\"{uid}\" type=\"b-m-p-s-p-loc\" how=\"m-g\" time=\"{t}\" start=\"{t}\" stale=\"{t}\">\
<point lat=\"{:.6}\" lon=\"{:.6}\" hae=\"{:.1}\" ce=\"9999999.0\" le=\"9999999.0\"/>\
{detail}</event>",
            self.lat, self.lon, self.hae
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CasTag {
    None,
    Line1,
    Line2,
    Line3,
    Line4,
    Line5,
    Line6,
    Line7,
    Line8,
    Line9,
    Remarks,
}

fn parse_f64(bytes: &[u8]) -> Option<f64> {
    std::str::from_utf8(bytes).ok()?.trim().parse().ok()
}

fn match_tag(name: &[u8]) -> CasTag {
    match name {
        b"line1" => CasTag::Line1,
        b"line2" => CasTag::Line2,
        b"line3" => CasTag::Line3,
        b"line4" => CasTag::Line4,
        b"line5" => CasTag::Line5,
        b"line6" => CasTag::Line6,
        b"line7" => CasTag::Line7,
        b"line8" => CasTag::Line8,
        b"line9" => CasTag::Line9,
        b"remarks" => CasTag::Remarks,
        _ => CasTag::None,
    }
}

fn parse_cas_attributes(e: &quick_xml::events::BytesStart, b: &mut CasNineLine) {
    for a in e.attributes().flatten() {
        let val = unescape_xml(&String::from_utf8_lossy(&a.value));
        match a.key.as_ref() {
            b"line1" => b.line1_ip_bp = val,
            b"line2" => b.line2_heading = val,
            b"line3" => b.line3_distance = val,
            b"line4" => b.line4_elevation = val,
            b"line5" => b.line5_description = val,
            b"line6" => b.line6_location = val,
            b"line7" => b.line7_mark_type = val,
            b"line8" => b.line8_friendlies = val,
            b"line9" => b.line9_egress = val,
            b"remarks" => b.remarks = val,
            b"jtac" => b.jtac_callsign = val,
            _ => {}
        }
    }
}

fn apply_text_to_field(tag: CasTag, text: &str, b: &mut CasNineLine) {
    let unescaped = unescape_xml(text);
    match tag {
        CasTag::Line1 => b.line1_ip_bp = unescaped,
        CasTag::Line2 => b.line2_heading = unescaped,
        CasTag::Line3 => b.line3_distance = unescaped,
        CasTag::Line4 => b.line4_elevation = unescaped,
        CasTag::Line5 => b.line5_description = unescaped,
        CasTag::Line6 => b.line6_location = unescaped,
        CasTag::Line7 => b.line7_mark_type = unescaped,
        CasTag::Line8 => b.line8_friendlies = unescaped,
        CasTag::Line9 => b.line9_egress = unescaped,
        CasTag::Remarks => b.remarks = unescaped,
        CasTag::None => {}
    }
}

fn parse_coord(val: &[u8], min: f64, max: f64) -> Result<f64, CasNineLineError> {
    parse_f64(val)
        .filter(|v| v.is_finite() && (min..=max).contains(v))
        .ok_or_else(|| CasNineLineError::InvalidCoordinate("invalid coord".into()))
}

fn parse_event_point(
    e: &quick_xml::events::BytesStart,
    b: &mut CasNineLine,
    name: &[u8],
) -> Result<(), CasNineLineError> {
    for a in e.attributes().flatten() {
        if name == b"event" {
            if a.key.as_ref() == b"uid" {
                b.cot_uid = unescape_xml(&String::from_utf8_lossy(&a.value));
            }
            if a.key.as_ref() == b"time" {
                b.time_iso = unescape_xml(&String::from_utf8_lossy(&a.value));
            }
        } else if name == b"point" {
            match a.key.as_ref() {
                b"lat" => b.lat = parse_coord(&a.value, -90.0, 90.0)?,
                b"lon" => b.lon = parse_coord(&a.value, -180.0, 180.0)?,
                b"hae" => b.hae = parse_coord(&a.value, f64::MIN, f64::MAX)?,
                _ => {}
            }
        } else if name == b"contact" && a.key.as_ref() == b"callsign" {
            b.jtac_callsign = unescape_xml(&String::from_utf8_lossy(&a.value));
        }
    }
    Ok(())
}

fn handle_start_empty(
    e: &quick_xml::events::BytesStart,
    res: &mut CasNineLine,
    flags: &mut (bool, bool, bool),
) -> Result<CasTag, CasNineLineError> {
    let name = e.name().into_inner();
    if name == b"event" {
        flags.0 = true;
    }
    if name == b"point" {
        flags.1 = true;
    }
    if name == b"cas9line" || name == b"cas" || name == b"nine_line" {
        flags.2 = true;
        parse_cas_attributes(e, res);
    }
    parse_event_point(e, res, name)?;
    Ok(match_tag(name))
}

fn run_parser_loop(
    mut reader: Reader<&[u8]>,
    res: &mut CasNineLine,
) -> Result<(bool, bool, bool), CasNineLineError> {
    let mut flags = (false, false, false);
    let mut tag = CasTag::None;
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                tag = handle_start_empty(e, res, &mut flags)?
            }
            Ok(Event::Text(ref e)) => {
                apply_text_to_field(tag, &String::from_utf8_lossy(e.as_ref()), res)
            }
            Ok(Event::End(_)) => tag = CasTag::None,
            Ok(Event::Eof) => break,
            Err(e) => return Err(CasNineLineError::InvalidXml(e.to_string())),
            _ => {}
        }
    }
    Ok(flags)
}

impl CasNineLine {
    /// Parses an ATAK CoT XML string into a CasNineLine.
    pub fn from_cot_xml(xml: &str) -> Result<Self, CasNineLineError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut res = Self::new("", "");
        let (found_event, found_point, found_cas) = run_parser_loop(reader, &mut res)?;
        if !found_event {
            return Err(CasNineLineError::MissingEventTag);
        }
        if !found_point {
            return Err(CasNineLineError::MissingPointTag);
        }
        if !found_cas {
            return Err(CasNineLineError::MissingCasTag);
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_brief() -> CasNineLine {
        let mut b = CasNineLine::new("CAS-001", "WARRIOR 21")
            .with_coordinates(34.123456, -117.654321, 1200.0)
            .with_time("2026-11-16T09:00:00Z")
            .with_remarks("FAH 180-210. Danger close 600m.");
        b.line1_ip_bp = "VIPER".to_string();
        b.line2_heading = "185".to_string();
        b.line3_distance = "12.4nm".to_string();
        b.line4_elevation = "3400MSL".to_string();
        b.line5_description = "2x T-72 tanks".to_string();
        b.line6_location = "38SMB12345678".to_string();
        b.line7_mark_type = "WP Smoke / Laser 1688".to_string();
        b.line8_friendlies = "1200m South".to_string();
        b.line9_egress = "Egress East to PULL".to_string();
        b
    }

    #[test]
    fn test_cas_nine_line_roundtrip_xml() {
        let b = sample_brief();
        let xml = b.to_cot_xml();
        assert!(xml.contains("<event version=\"2.0\""));
        assert!(xml.contains("uid=\"CAS-001\""));
        assert!(xml.contains("<cas9line"));
        let parsed = CasNineLine::from_cot_xml(&xml).expect("parse failed");
        assert_eq!(parsed.cot_uid, b.cot_uid);
        assert_eq!(parsed.jtac_callsign, b.jtac_callsign);
        assert_eq!(parsed.line1_ip_bp, b.line1_ip_bp);
        assert_eq!(parsed.line5_description, b.line5_description);
        assert_eq!(parsed.line6_location, b.line6_location);
        assert_eq!(parsed.remarks, b.remarks);
        assert!((parsed.lat - b.lat).abs() < 1e-5);
    }

    #[test]
    fn test_format_cas_nine_line_text() {
        let b = sample_brief();
        let txt = format_cas_nine_line_text(&b);
        assert!(txt.contains("--- 9-LINE CAS BRIEF ---"));
        assert!(txt.contains("1. IP/BP: VIPER"));
        assert!(txt.contains("2. Heading: 185"));
        assert!(txt.contains("3. Distance: 12.4nm"));
        assert!(txt.contains("4. Elevation: 3400MSL"));
        assert!(txt.contains("5. Description: 2x T-72 tanks"));
        assert!(txt.contains("6. Location: 38SMB12345678"));
        assert!(txt.contains("7. Mark: WP Smoke / Laser 1688"));
        assert!(txt.contains("8. Friendlies: 1200m South"));
        assert!(txt.contains("9. Egress: Egress East to PULL"));
        assert!(txt.contains("Remarks: FAH 180-210"));
    }

    #[test]
    fn test_parse_cas_child_tags() {
        let xml = "<event version=\"2.0\" uid=\"CAS-002\" time=\"2026-11-16T10:00:00Z\">\
<point lat=\"32.0\" lon=\"-115.0\" hae=\"500.0\"/>\
<detail><contact callsign=\"HAWK 11\"/>\
<cas9line><line1>EAGLE</line1><line5>Bunker</line5><line6>38SMC99887766</line6>\
<remarks>Type 2 Control</remarks></cas9line></detail></event>";
        let parsed = CasNineLine::from_cot_xml(xml).expect("parse child tags failed");
        assert_eq!(parsed.cot_uid, "CAS-002");
        assert_eq!(parsed.jtac_callsign, "HAWK 11");
        assert_eq!(parsed.line1_ip_bp, "EAGLE");
        assert_eq!(parsed.line5_description, "Bunker");
        assert_eq!(parsed.line6_location, "38SMC99887766");
        assert_eq!(parsed.remarks, "Type 2 Control");
    }

    #[test]
    fn test_cas_parser_errors() {
        let no_event = "<data><point lat=\"0\" lon=\"0\"/></data>";
        assert_eq!(
            CasNineLine::from_cot_xml(no_event).unwrap_err(),
            CasNineLineError::MissingEventTag
        );

        let no_point = "<event uid=\"1\"><detail><cas9line/></detail></event>";
        assert_eq!(
            CasNineLine::from_cot_xml(no_point).unwrap_err(),
            CasNineLineError::MissingPointTag
        );

        let no_cas = "<event uid=\"1\"><point lat=\"0\" lon=\"0\"/><detail></detail></event>";
        assert_eq!(
            CasNineLine::from_cot_xml(no_cas).unwrap_err(),
            CasNineLineError::MissingCasTag
        );
    }

    #[test]
    fn test_xml_escaping() {
        let mut b = CasNineLine::new("CAS-ESC", "VIPER & CO");
        b.line5_description = "HQ <Command> & 'Bunker'".to_string();
        b.remarks = "Watch out for \"SAM\" & AAA".to_string();
        let xml = b.to_cot_xml();
        assert!(xml.contains("VIPER &amp; CO"));
        assert!(xml.contains("&lt;Command&gt;"));
        let parsed = CasNineLine::from_cot_xml(&xml).expect("parse escaped xml failed");
        assert_eq!(parsed.jtac_callsign, "VIPER & CO");
        assert_eq!(parsed.line5_description, "HQ <Command> & 'Bunker'");
        assert_eq!(parsed.remarks, "Watch out for \"SAM\" & AAA");
    }
}
