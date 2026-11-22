//! CoT emergency casualty evacuation (CASEVAC / 9-Line MEDEVAC) builder and parser.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors during 9-Line MEDEVAC serialization and parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MedevacError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingMedevacTag,
    InvalidCoordinate(String),
}

impl fmt::Display for MedevacError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingMedevacTag => write!(f, "Missing medevac detail tag"),
            Self::InvalidCoordinate(e) => write!(f, "Invalid coordinate: {e}"),
        }
    }
}

impl std::error::Error for MedevacError {}

/// Line 3: Patient evacuation precedence category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MedevacPrecedence {
    Urgent,
    UrgentSurgical,
    #[default]
    Priority,
    Routine,
    Convenience,
}

impl MedevacPrecedence {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Urgent => "A - URGENT",
            Self::UrgentSurgical => "B - URGENT-SURG",
            Self::Priority => "C - PRIORITY",
            Self::Routine => "D - ROUTINE",
            Self::Convenience => "E - CONVENIENCE",
        }
    }

    pub fn target_evacuation_window_hours(&self) -> Option<u32> {
        match self {
            Self::Urgent | Self::UrgentSurgical => Some(2),
            Self::Priority => Some(4),
            Self::Routine => Some(24),
            Self::Convenience => None,
        }
    }

    pub fn parse_code(s: &str) -> Option<Self> {
        let upper = s.trim().to_uppercase();
        if upper.starts_with('E') || upper.contains("CONVENIENCE") {
            Some(Self::Convenience)
        } else if upper.starts_with('A') || (upper.contains("URGENT") && !upper.contains("SURG")) {
            Some(Self::Urgent)
        } else if upper.starts_with('B') || upper.contains("SURG") {
            Some(Self::UrgentSurgical)
        } else if upper.starts_with('C') || upper.contains("PRIORITY") {
            Some(Self::Priority)
        } else if upper.starts_with('D') || upper.contains("ROUTINE") {
            Some(Self::Routine)
        } else {
            None
        }
    }
}

/// Line 4: Special equipment requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MedevacEquipment {
    #[default]
    None,
    Hoist,
    ExtractionEquipment,
    Ventilator,
    Custom(String),
}

impl MedevacEquipment {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "None",
            Self::Hoist => "Hoist",
            Self::ExtractionEquipment => "Extraction Equipment",
            Self::Ventilator => "Ventilator",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn from_description(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "" | "none" => Self::None,
            "hoist" => Self::Hoist,
            "extraction" | "extraction equipment" => Self::ExtractionEquipment,
            "ventilator" => Self::Ventilator,
            _ => Self::Custom(s.trim().to_string()),
        }
    }
}

/// Line 6: Security at pickup site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SiteSecurity {
    #[default]
    NoEnemy,
    PossibleEnemy,
    EnemyInArea,
    ArmedEscortRequired,
    Custom(String),
}

impl SiteSecurity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::NoEnemy => "N - No enemy troops",
            Self::PossibleEnemy => "P - Possible enemy troops",
            Self::EnemyInArea => "E - Enemy troops in area",
            Self::ArmedEscortRequired => "X - Armed escort required",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn parse_security(s: &str) -> Self {
        let upper = s.trim().to_uppercase();
        if upper.starts_with('N') {
            Self::NoEnemy
        } else if upper.starts_with('P') {
            Self::PossibleEnemy
        } else if upper.starts_with('E') {
            Self::EnemyInArea
        } else if upper.starts_with('X') {
            Self::ArmedEscortRequired
        } else {
            Self::Custom(s.trim().to_string())
        }
    }
}

/// Line 7: Helicopter landing zone marking method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HlzMarking {
    Panels,
    PyrotechnicSignal,
    Smoke(String),
    #[default]
    None,
    Custom(String),
}

impl HlzMarking {
    pub fn as_str(&self) -> String {
        match self {
            Self::Panels => "Panels".to_string(),
            Self::PyrotechnicSignal => "Pyrotechnic Signal".to_string(),
            Self::Smoke(color) => format!("Smoke ({color})"),
            Self::None => "None".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }

    pub fn parse_marking(s: &str) -> Self {
        let trimmed = s.trim();
        let lower = trimmed.to_lowercase();
        if lower.contains("panel") {
            Self::Panels
        } else if lower.contains("pyro") {
            Self::PyrotechnicSignal
        } else if let Some(idx) = lower.find("smoke") {
            let rest = trimmed[idx + 5..].trim();
            let col = rest
                .strip_prefix('(')
                .and_then(|r| r.strip_suffix(')'))
                .unwrap_or(rest)
                .trim();
            Self::Smoke(if col.is_empty() {
                trimmed.to_string()
            } else {
                col.to_string()
            })
        } else if lower == "none" || lower.is_empty() {
            Self::None
        } else {
            Self::Custom(trimmed.to_string())
        }
    }
}

/// Line 8: Patient nationality and status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PatientNationality {
    #[default]
    UsMilitary,
    UsCivilian,
    NonUsMilitary,
    NonUsCivilian,
    Epw,
    Child,
    Custom(String),
}

impl PatientNationality {
    pub fn as_str(&self) -> &str {
        match self {
            Self::UsMilitary => "US Military",
            Self::UsCivilian => "US Civilian",
            Self::NonUsMilitary => "Non-US Military",
            Self::NonUsCivilian => "Non-US Civilian",
            Self::Epw => "EPW / Detainee",
            Self::Child => "Child",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn parse_nationality(s: &str) -> Self {
        let upper = s.trim().to_uppercase();
        if upper.contains("NON-US MIL") {
            Self::NonUsMilitary
        } else if upper.contains("NON-US CIV") {
            Self::NonUsCivilian
        } else if upper.contains("US MIL") {
            Self::UsMilitary
        } else if upper.contains("US CIV") {
            Self::UsCivilian
        } else if upper.contains("EPW") || upper.contains("DETAIN") {
            Self::Epw
        } else if upper.contains("CHILD") {
            Self::Child
        } else {
            Self::Custom(s.trim().to_string())
        }
    }
}

/// Line 9: Nuclear, Biological, Chemical (NBC) contamination status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NbcContamination {
    #[default]
    None,
    Nuclear,
    Biological,
    Chemical,
    Custom(String),
}

impl NbcContamination {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "None",
            Self::Nuclear => "Nuclear",
            Self::Biological => "Biological",
            Self::Chemical => "Chemical",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn parse_nbc(s: &str) -> Self {
        let upper = s.trim().to_uppercase();
        if upper.starts_with("N - NONE") || upper == "NONE" || upper.is_empty() {
            Self::None
        } else if upper.starts_with("N - NUC") || upper.contains("NUCLEAR") {
            Self::Nuclear
        } else if upper.starts_with('B') || upper.contains("BIO") {
            Self::Biological
        } else if upper.starts_with('C') || upper.contains("CHEM") {
            Self::Chemical
        } else {
            Self::Custom(s.trim().to_string())
        }
    }
}

/// NATO 9-Line MEDEVAC and CASEVAC incident report packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CotMedevacReport {
    pub cot_uid: String,
    pub callsign: String,
    pub cot_type: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub time_iso: String,
    pub line1_location: String,
    pub line2_freq_callsign: String,
    pub line3_precedence: MedevacPrecedence,
    pub line3_urgent_count: u32,
    pub line3_priority_count: u32,
    pub line3_routine_count: u32,
    pub line4_equipment: MedevacEquipment,
    pub line5_litter_count: u32,
    pub line5_ambulatory_count: u32,
    pub line6_security: SiteSecurity,
    pub line7_hlz_marking: HlzMarking,
    pub line8_patient_nationality: PatientNationality,
    pub line9_terrain_nbc: String,
    pub remarks: String,
}

impl CotMedevacReport {
    pub fn total_patients_by_precedence(&self) -> u32 {
        self.line3_urgent_count + self.line3_priority_count + self.line3_routine_count
    }

    pub fn total_patients_by_type(&self) -> u32 {
        self.line5_litter_count + self.line5_ambulatory_count
    }

    pub fn is_urgent(&self) -> bool {
        matches!(
            self.line3_precedence,
            MedevacPrecedence::Urgent | MedevacPrecedence::UrgentSurgical
        ) || self.line3_urgent_count > 0
    }

    pub fn requires_special_equipment(&self) -> bool {
        !matches!(self.line4_equipment, MedevacEquipment::None)
    }

    pub fn requires_armed_escort(&self) -> bool {
        matches!(self.line6_security, SiteSecurity::ArmedEscortRequired)
    }

    pub fn nbc_contamination(&self) -> NbcContamination {
        NbcContamination::parse_nbc(&self.line9_terrain_nbc)
    }
}

/// Builder for constructing validated 9-Line MEDEVAC reports.
#[derive(Debug, Clone, Default)]
pub struct CotMedevacBuilder {
    cot_uid: String,
    callsign: String,
    cot_type: String,
    lat: f64,
    lon: f64,
    hae: f64,
    time_iso: String,
    line1_location: String,
    line2_freq_callsign: String,
    line3_precedence: MedevacPrecedence,
    line3_urgent_count: u32,
    line3_priority_count: u32,
    line3_routine_count: u32,
    line4_equipment: MedevacEquipment,
    line5_litter_count: u32,
    line5_ambulatory_count: u32,
    line6_security: SiteSecurity,
    line7_hlz_marking: HlzMarking,
    line8_patient_nationality: PatientNationality,
    line9_terrain_nbc: String,
    remarks: String,
}

impl CotMedevacBuilder {
    pub fn new(cot_uid: impl Into<String>, callsign: impl Into<String>) -> Self {
        Self {
            cot_uid: cot_uid.into(),
            callsign: callsign.into(),
            cot_type: "b-r-f-h-c".to_string(),
            time_iso: "1970-01-01T00:00:00Z".to_string(),
            ..Default::default()
        }
    }

    pub fn coordinates(mut self, lat: f64, lon: f64, hae: f64) -> Self {
        self.lat = lat;
        self.lon = lon;
        self.hae = hae;
        self
    }

    pub fn line1_location(mut self, location: impl Into<String>) -> Self {
        self.line1_location = location.into();
        self
    }

    pub fn line2_freq_callsign(mut self, freq: impl Into<String>) -> Self {
        self.line2_freq_callsign = freq.into();
        self
    }

    pub fn patient_counts(
        mut self,
        urgent: u32,
        priority: u32,
        routine: u32,
        litter: u32,
        ambulatory: u32,
    ) -> Self {
        self.line3_urgent_count = urgent;
        self.line3_priority_count = priority;
        self.line3_routine_count = routine;
        self.line5_litter_count = litter;
        self.line5_ambulatory_count = ambulatory;
        self
    }

    pub fn precedence(mut self, precedence: MedevacPrecedence) -> Self {
        self.line3_precedence = precedence;
        self
    }

    pub fn equipment(mut self, equipment: MedevacEquipment) -> Self {
        self.line4_equipment = equipment;
        self
    }

    pub fn security(mut self, security: SiteSecurity) -> Self {
        self.line6_security = security;
        self
    }

    pub fn marking(mut self, marking: HlzMarking) -> Self {
        self.line7_hlz_marking = marking;
        self
    }

    pub fn nationality(mut self, nationality: PatientNationality) -> Self {
        self.line8_patient_nationality = nationality;
        self
    }

    pub fn terrain_nbc(mut self, terrain_nbc: impl Into<String>) -> Self {
        self.line9_terrain_nbc = terrain_nbc.into();
        self
    }

    pub fn nbc(mut self, contamination: NbcContamination) -> Self {
        self.line9_terrain_nbc = contamination.as_str().to_string();
        self
    }

    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = remarks.into();
        self
    }

    pub fn time_iso(mut self, time_iso: impl Into<String>) -> Self {
        self.time_iso = time_iso.into();
        self
    }

    fn validate_coordinates(&self) -> Result<(), MedevacError> {
        if !self.lat.is_finite() || !(-90.0..=90.0).contains(&self.lat) {
            return Err(MedevacError::InvalidCoordinate(format!(
                "Latitude {} out of [-90, 90]",
                self.lat
            )));
        }
        if !self.lon.is_finite() || !(-180.0..=180.0).contains(&self.lon) {
            return Err(MedevacError::InvalidCoordinate(format!(
                "Longitude {} out of [-180, 180]",
                self.lon
            )));
        }
        if !self.hae.is_finite() {
            return Err(MedevacError::InvalidCoordinate(format!(
                "HAE {} is non-finite",
                self.hae
            )));
        }
        Ok(())
    }

    fn into_report(self) -> CotMedevacReport {
        CotMedevacReport {
            cot_uid: self.cot_uid,
            callsign: self.callsign,
            cot_type: self.cot_type,
            lat: self.lat,
            lon: self.lon,
            hae: self.hae,
            time_iso: self.time_iso,
            line1_location: self.line1_location,
            line2_freq_callsign: self.line2_freq_callsign,
            line3_precedence: self.line3_precedence,
            line3_urgent_count: self.line3_urgent_count,
            line3_priority_count: self.line3_priority_count,
            line3_routine_count: self.line3_routine_count,
            line4_equipment: self.line4_equipment,
            line5_litter_count: self.line5_litter_count,
            line5_ambulatory_count: self.line5_ambulatory_count,
            line6_security: self.line6_security,
            line7_hlz_marking: self.line7_hlz_marking,
            line8_patient_nationality: self.line8_patient_nationality,
            line9_terrain_nbc: self.line9_terrain_nbc,
            remarks: self.remarks,
        }
    }

    pub fn build(self) -> Result<CotMedevacReport, MedevacError> {
        self.validate_coordinates()?;
        Ok(self.into_report())
    }
}

/// Formats standard NATO 9-Line MEDEVAC textual voice readback brief.
pub fn format_nine_line_voice_brief(r: &CotMedevacReport) -> String {
    format!(
        "--- 9-LINE MEDEVAC BRIEF ---\n\
         LINE 1 (Location): {}\n\
         LINE 2 (Frequency / Callsign): {}\n\
         LINE 3 (Precedence): {} (U:{} P:{} R:{})\n\
         LINE 4 (Special Equipment): {}\n\
         LINE 5 (Patients): {} Litter / {} Ambulatory\n\
         LINE 6 (Security): {}\n\
         LINE 7 (Marking): {}\n\
         LINE 8 (Nationality): {}\n\
         LINE 9 (Terrain / NBC): {}\n\
         Remarks: {}",
        r.line1_location,
        r.line2_freq_callsign,
        r.line3_precedence.code(),
        r.line3_urgent_count,
        r.line3_priority_count,
        r.line3_routine_count,
        r.line4_equipment.as_str(),
        r.line5_litter_count,
        r.line5_ambulatory_count,
        r.line6_security.as_str(),
        r.line7_hlz_marking.as_str(),
        r.line8_patient_nationality.as_str(),
        r.line9_terrain_nbc,
        r.remarks
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

fn build_medevac_inner_tags(r: &CotMedevacReport) -> String {
    let (l1, l2, l3, l4) = (
        escape_xml(&r.line1_location),
        escape_xml(&r.line2_freq_callsign),
        escape_xml(r.line3_precedence.code()),
        escape_xml(r.line4_equipment.as_str()),
    );
    let (l6, l7, l8, l9) = (
        escape_xml(r.line6_security.as_str()),
        escape_xml(&r.line7_hlz_marking.as_str()),
        escape_xml(r.line8_patient_nationality.as_str()),
        escape_xml(&r.line9_terrain_nbc),
    );
    format!(
        "<line1>{l1}</line1><line2>{l2}</line2><line3>{l3}</line3><line4>{l4}</line4>\
<urgent>{}</urgent><priority>{}</priority><routine>{}</routine>\
<litter>{}</litter><ambulatory>{}</ambulatory>\
<line6>{l6}</line6><line7>{l7}</line7><line8>{l8}</line8><line9>{l9}</line9>",
        r.line3_urgent_count,
        r.line3_priority_count,
        r.line3_routine_count,
        r.line5_litter_count,
        r.line5_ambulatory_count
    )
}

fn format_medevac_detail_xml(r: &CotMedevacReport) -> String {
    let (cs, rem) = (escape_xml(&r.callsign), escape_xml(&r.remarks));
    let (l1, l2) = (
        escape_xml(&r.line1_location),
        escape_xml(&r.line2_freq_callsign),
    );
    let (l3, l4) = (
        escape_xml(r.line3_precedence.code()),
        escape_xml(r.line4_equipment.as_str()),
    );
    let (l6, l7) = (
        escape_xml(r.line6_security.as_str()),
        escape_xml(&r.line7_hlz_marking.as_str()),
    );
    let (l8, l9) = (
        escape_xml(r.line8_patient_nationality.as_str()),
        escape_xml(&r.line9_terrain_nbc),
    );
    let inner = build_medevac_inner_tags(r);
    format!(
        "<detail><contact callsign=\"{cs}\"/><remarks>{rem}</remarks>\
<medevac line1=\"{l1}\" line2=\"{l2}\" line3=\"{l3}\" line4=\"{l4}\" \
urgent=\"{}\" priority=\"{}\" routine=\"{}\" \
litter=\"{}\" ambulatory=\"{}\" \
line6=\"{l6}\" line7=\"{l7}\" line8=\"{l8}\" line9=\"{l9}\">{inner}</medevac></detail>",
        r.line3_urgent_count,
        r.line3_priority_count,
        r.line3_routine_count,
        r.line5_litter_count,
        r.line5_ambulatory_count
    )
}

impl CotMedevacReport {
    /// Serializes 9-Line MEDEVAC report to standard Cursor-on-Target XML string.
    pub fn to_cot_xml(&self) -> String {
        let (uid, typ, time) = (
            escape_xml(&self.cot_uid),
            escape_xml(&self.cot_type),
            escape_xml(&self.time_iso),
        );
        let stale = escape_xml(&Self::derive_stale_iso(&self.time_iso));
        let detail = format_medevac_detail_xml(self);
        format!(
            "<event version=\"2.0\" uid=\"{uid}\" type=\"{typ}\" \
time=\"{time}\" start=\"{time}\" stale=\"{stale}\" how=\"m-g\">\
<point lat=\"{:.6}\" lon=\"{:.6}\" \
hae=\"{:.1}\" ce=\"9999999\" le=\"9999999\"/>\
{detail}</event>",
            self.lat, self.lon, self.hae
        )
    }

    fn derive_stale_iso(time_iso: &str) -> String {
        let b = time_iso.as_bytes();
        if b.len() == 20
            && b[4] == b'-'
            && b[7] == b'-'
            && b[10] == b'T'
            && b[13] == b':'
            && b[16] == b':'
            && b[19] == b'Z'
        {
            let (mut y, mut mo, mut d) = (
                time_iso[0..4].parse::<i64>().unwrap_or(0),
                time_iso[5..7].parse::<i64>().unwrap_or(0),
                time_iso[8..10].parse::<i64>().unwrap_or(0),
            );
            let (mut h, mut mi, s) = (
                time_iso[11..13].parse::<i64>().unwrap_or(0),
                time_iso[14..16].parse::<i64>().unwrap_or(0),
                time_iso[17..19].parse::<i64>().unwrap_or(0),
            );
            if (1..=12).contains(&mo) && (1..=31).contains(&d) && h < 24 && mi < 60 && s < 60 {
                mi += 5;
                if mi >= 60 {
                    mi -= 60;
                    h += 1;
                    if h >= 24 {
                        h -= 24;
                        d += 1;
                        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
                        let dim = [
                            31,
                            if leap { 29 } else { 28 },
                            31,
                            30,
                            31,
                            30,
                            31,
                            31,
                            30,
                            31,
                            30,
                            31,
                        ];
                        if d > dim[(mo - 1) as usize] {
                            d = 1;
                            mo += 1;
                            if mo > 12 {
                                mo = 1;
                                y += 1;
                            }
                        }
                    }
                }
                return format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z");
            }
        }
        time_iso.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MedevacTag {
    None,
    Line1,
    Line2,
    Line3,
    Line4,
    Urgent,
    Priority,
    Routine,
    Litter,
    Ambulatory,
    Line6,
    Line7,
    Line8,
    Line9,
    Remarks,
}

fn parse_f64(bytes: &[u8]) -> Option<f64> {
    std::str::from_utf8(bytes).ok()?.trim().parse().ok()
}

fn match_medevac_tag(name: &[u8]) -> MedevacTag {
    match name {
        b"line1" => MedevacTag::Line1,
        b"line2" => MedevacTag::Line2,
        b"line3" => MedevacTag::Line3,
        b"line4" => MedevacTag::Line4,
        b"urgent" => MedevacTag::Urgent,
        b"priority" => MedevacTag::Priority,
        b"routine" => MedevacTag::Routine,
        b"litter" => MedevacTag::Litter,
        b"ambulatory" => MedevacTag::Ambulatory,
        b"line6" => MedevacTag::Line6,
        b"line7" => MedevacTag::Line7,
        b"line8" => MedevacTag::Line8,
        b"line9" => MedevacTag::Line9,
        b"remarks" => MedevacTag::Remarks,
        _ => MedevacTag::None,
    }
}

fn parse_medevac_attributes(e: &quick_xml::events::BytesStart, r: &mut CotMedevacReport) {
    for a in e.attributes().flatten() {
        let val = unescape_xml(&String::from_utf8_lossy(&a.value));
        match a.key.as_ref() {
            b"line1" => r.line1_location = val,
            b"line2" => r.line2_freq_callsign = val,
            b"line3" => {
                r.line3_precedence = MedevacPrecedence::parse_code(&val).unwrap_or_default()
            }
            b"line4" => r.line4_equipment = MedevacEquipment::from_description(&val),
            b"urgent" => r.line3_urgent_count = val.parse().unwrap_or(0),
            b"priority" => r.line3_priority_count = val.parse().unwrap_or(0),
            b"routine" => r.line3_routine_count = val.parse().unwrap_or(0),
            b"litter" => r.line5_litter_count = val.parse().unwrap_or(0),
            b"ambulatory" => r.line5_ambulatory_count = val.parse().unwrap_or(0),
            b"line6" => r.line6_security = SiteSecurity::parse_security(&val),
            b"line7" => r.line7_hlz_marking = HlzMarking::parse_marking(&val),
            b"line8" => r.line8_patient_nationality = PatientNationality::parse_nationality(&val),
            b"line9" => r.line9_terrain_nbc = val,
            _ => {}
        }
    }
}

fn apply_text_to_medevac(tag: MedevacTag, text: &str, r: &mut CotMedevacReport) {
    let val = text.to_string();
    match tag {
        MedevacTag::Line1 => r.line1_location = val,
        MedevacTag::Line2 => r.line2_freq_callsign = val,
        MedevacTag::Line3 => {
            r.line3_precedence = MedevacPrecedence::parse_code(&val).unwrap_or_default()
        }
        MedevacTag::Line4 => r.line4_equipment = MedevacEquipment::from_description(&val),
        MedevacTag::Urgent => r.line3_urgent_count = val.parse().unwrap_or(0),
        MedevacTag::Priority => r.line3_priority_count = val.parse().unwrap_or(0),
        MedevacTag::Routine => r.line3_routine_count = val.parse().unwrap_or(0),
        MedevacTag::Litter => r.line5_litter_count = val.parse().unwrap_or(0),
        MedevacTag::Ambulatory => r.line5_ambulatory_count = val.parse().unwrap_or(0),
        MedevacTag::Line6 => r.line6_security = SiteSecurity::parse_security(&val),
        MedevacTag::Line7 => r.line7_hlz_marking = HlzMarking::parse_marking(&val),
        MedevacTag::Line8 => {
            r.line8_patient_nationality = PatientNationality::parse_nationality(&val)
        }
        MedevacTag::Line9 => r.line9_terrain_nbc = val,
        MedevacTag::Remarks => r.remarks = val,
        MedevacTag::None => {}
    }
}

fn parse_coord(val: &[u8], min: f64, max: f64) -> Result<f64, MedevacError> {
    parse_f64(val)
        .filter(|v| v.is_finite() && (min..=max).contains(v))
        .ok_or_else(|| MedevacError::InvalidCoordinate("invalid coord".into()))
}

fn parse_event_point(
    e: &quick_xml::events::BytesStart,
    r: &mut CotMedevacReport,
    name: &[u8],
) -> Result<(), MedevacError> {
    for a in e.attributes().flatten() {
        let val = unescape_xml(&String::from_utf8_lossy(&a.value));
        match (name, a.key.as_ref()) {
            (b"event", b"uid") => r.cot_uid = val,
            (b"event", b"type") => r.cot_type = val,
            (b"event", b"time") => r.time_iso = val,
            (b"point", b"lat") => r.lat = parse_coord(&a.value, -90.0, 90.0)?,
            (b"point", b"lon") => r.lon = parse_coord(&a.value, -180.0, 180.0)?,
            (b"point", b"hae") => r.hae = parse_coord(&a.value, f64::MIN, f64::MAX)?,
            (b"contact", b"callsign") => r.callsign = val,
            _ => {}
        }
    }
    Ok(())
}

fn handle_start_empty(
    e: &quick_xml::events::BytesStart,
    r: &mut CotMedevacReport,
    cur: &mut MedevacTag,
    has_event: &mut bool,
    has_point: &mut bool,
    has_medevac: &mut bool,
) -> Result<(), MedevacError> {
    let name = e.name().into_inner();
    match name {
        b"event" => {
            *has_event = true;
            parse_event_point(e, r, name)?;
        }
        b"point" => {
            *has_point = true;
            parse_event_point(e, r, name)?;
        }
        b"medevac" | b"casevac" => {
            *has_medevac = true;
            parse_medevac_attributes(e, r);
        }
        b"contact" => parse_event_point(e, r, name)?,
        _ => *cur = match_medevac_tag(name),
    }
    Ok(())
}

fn run_medevac_parser_loop(
    reader: &mut Reader<&[u8]>,
    r: &mut CotMedevacReport,
    buf: &mut Vec<u8>,
    ev: &mut bool,
    pt: &mut bool,
    md: &mut bool,
) -> Result<(), MedevacError> {
    let mut cur = MedevacTag::None;
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => handle_start_empty(e, r, &mut cur, ev, pt, md)?,
            Ok(Event::Empty(ref e)) => handle_start_empty(e, r, &mut cur, ev, pt, md)?,
            Ok(Event::Text(ref e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| MedevacError::InvalidXml(e.to_string()))?;
                apply_text_to_medevac(cur, &text, r);
            }
            Ok(Event::End(_)) => cur = MedevacTag::None,
            Ok(Event::Eof) => break,
            Err(e) => return Err(MedevacError::InvalidXml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

impl CotMedevacReport {
    /// Parses 9-Line MEDEVAC report from Cursor-on-Target XML representation.
    pub fn from_cot_xml(xml: &str) -> Result<Self, MedevacError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut r = CotMedevacBuilder::new("", "").build()?;
        let mut buf = Vec::new();
        let (mut ev, mut pt, mut md) = (false, false, false);
        run_medevac_parser_loop(&mut reader, &mut r, &mut buf, &mut ev, &mut pt, &mut md)?;
        if !ev {
            return Err(MedevacError::MissingEventTag);
        }
        if !pt {
            return Err(MedevacError::MissingPointTag);
        }
        if !md {
            return Err(MedevacError::MissingMedevacTag);
        }
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_medevac_report() -> CotMedevacReport {
        CotMedevacBuilder::new("MED-101", "DUSTOFF-42")
            .coordinates(34.55, 43.12, 120.0)
            .line1_location("38SMB 1234 5678")
            .line2_freq_callsign("42.50 MHz DUSTOFF")
            .precedence(MedevacPrecedence::Urgent)
            .patient_counts(2, 1, 0, 2, 1)
            .equipment(MedevacEquipment::Hoist)
            .security(SiteSecurity::PossibleEnemy)
            .marking(HlzMarking::Smoke("Green".into()))
            .nationality(PatientNationality::UsMilitary)
            .terrain_nbc("Flat LZ, wires east")
            .remarks("Urgent evac needed")
            .build()
            .unwrap()
    }

    #[test]
    fn test_precedence_and_window() {
        let p = MedevacPrecedence::Urgent;
        assert_eq!(p.code(), "A - URGENT");
        assert_eq!(p.target_evacuation_window_hours(), Some(2));
        assert_eq!(
            MedevacPrecedence::parse_code("B - URGENT-SURG"),
            Some(MedevacPrecedence::UrgentSurgical)
        );
        assert_eq!(
            MedevacPrecedence::parse_code("routine"),
            Some(MedevacPrecedence::Routine)
        );
        assert_eq!(
            MedevacPrecedence::Convenience.target_evacuation_window_hours(),
            None
        );
    }

    #[test]
    fn test_enums_string_roundtrip() {
        let eq = MedevacEquipment::from_description("hoist");
        assert_eq!(eq, MedevacEquipment::Hoist);
        assert_eq!(eq.as_str(), "Hoist");

        let sec = SiteSecurity::parse_security("X - Armed escort required");
        assert_eq!(sec, SiteSecurity::ArmedEscortRequired);
        assert_eq!(sec.as_str(), "X - Armed escort required");

        let hlz = HlzMarking::parse_marking("Smoke (Green)");
        assert_eq!(hlz, HlzMarking::Smoke("Green".into()));

        let nat = PatientNationality::parse_nationality("US MILITARY");
        assert_eq!(nat, PatientNationality::UsMilitary);

        let nbc = NbcContamination::parse_nbc("B - Biological");
        assert_eq!(nbc, NbcContamination::Biological);
        assert_eq!(nbc.as_str(), "Biological");
    }

    #[test]
    fn test_medevac_builder_valid_and_metrics() {
        let report = sample_medevac_report();
        assert_eq!(report.cot_uid, "MED-101");
        assert_eq!(report.callsign, "DUSTOFF-42");
        assert_eq!(report.total_patients_by_precedence(), 3);
        assert_eq!(report.total_patients_by_type(), 3);
        assert!(report.is_urgent());
        assert!(report.requires_special_equipment());
        assert!(!report.requires_armed_escort());
    }

    #[test]
    fn test_medevac_builder_validation_errors() {
        let bad_lat = CotMedevacBuilder::new("1", "A")
            .coordinates(95.0, 10.0, 0.0)
            .build();
        assert!(matches!(bad_lat, Err(MedevacError::InvalidCoordinate(_))));

        let bad_lon = CotMedevacBuilder::new("1", "A")
            .coordinates(10.0, -195.0, 0.0)
            .build();
        assert!(matches!(bad_lon, Err(MedevacError::InvalidCoordinate(_))));
    }

    #[test]
    fn test_format_nine_line_voice_brief() {
        let report = sample_medevac_report();
        let brief = format_nine_line_voice_brief(&report);
        assert!(brief.contains("--- 9-LINE MEDEVAC BRIEF ---"));
        assert!(brief.contains("LINE 1 (Location): 38SMB 1234 5678"));
        assert!(brief.contains("LINE 2 (Frequency / Callsign): 42.50 MHz DUSTOFF"));
        assert!(brief.contains("LINE 3 (Precedence): A - URGENT (U:2 P:1 R:0)"));
        assert!(brief.contains("LINE 4 (Special Equipment): Hoist"));
        assert!(brief.contains("LINE 5 (Patients): 2 Litter / 1 Ambulatory"));
        assert!(brief.contains("LINE 6 (Security): P - Possible enemy troops"));
        assert!(brief.contains("Smoke (Green)"));
        assert!(brief.contains("Flat LZ, wires east"));
    }

    #[test]
    fn test_medevac_cot_xml_roundtrip() {
        let report = sample_medevac_report();
        let xml = report.to_cot_xml();
        assert!(xml.contains("<event version=\"2.0\" uid=\"MED-101\""));
        assert!(xml.contains("<point lat=\"34.550000\" lon=\"43.120000\""));
        assert!(xml.contains("<contact callsign=\"DUSTOFF-42\""));
        assert!(xml.contains("<medevac "));

        let parsed = CotMedevacReport::from_cot_xml(&xml).expect("parse xml");
        assert_eq!(parsed.cot_uid, report.cot_uid);
        assert_eq!(parsed.callsign, report.callsign);
        assert_eq!(parsed.line1_location, report.line1_location);
        assert_eq!(parsed.line3_urgent_count, 2);
        assert_eq!(parsed.line5_litter_count, 2);
    }

    #[test]
    fn test_medevac_parser_errors() {
        let no_event = "<data><point lat=\"0\" lon=\"0\"/></data>";
        assert_eq!(
            CotMedevacReport::from_cot_xml(no_event).unwrap_err(),
            MedevacError::MissingEventTag
        );
        let no_point = "<event version=\"2.0\" uid=\"M\"><detail><medevac/></detail></event>";
        assert_eq!(
            CotMedevacReport::from_cot_xml(no_point).unwrap_err(),
            MedevacError::MissingPointTag
        );
        let no_medevac = "<event version=\"2.0\" uid=\"M\"><point lat=\"0\" lon=\"0\" hae=\"0\"/><detail/></event>";
        assert_eq!(
            CotMedevacReport::from_cot_xml(no_medevac).unwrap_err(),
            MedevacError::MissingMedevacTag
        );
    }
}
