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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MedevacPrecedence {
    Urgent,
    UrgentSurgical,
    Priority,
    Routine,
    Convenience,
}

impl Default for MedevacPrecedence {
    fn default() -> Self {
        Self::Priority
    }
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
        if upper.starts_with('A') || (upper.contains("URGENT") && !upper.contains("SURG")) {
            Some(Self::Urgent)
        } else if upper.starts_with('B') || upper.contains("SURG") {
            Some(Self::UrgentSurgical)
        } else if upper.starts_with('C') || upper.contains("PRIORITY") {
            Some(Self::Priority)
        } else if upper.starts_with('D') || upper.contains("ROUTINE") {
            Some(Self::Routine)
        } else if upper.starts_with('E') || upper.contains("CONVENIENCE") {
            Some(Self::Convenience)
        } else {
            None
        }
    }
}

/// Line 4: Special equipment requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MedevacEquipment {
    None,
    Hoist,
    ExtractionEquipment,
    Ventilator,
    Custom(String),
}

impl Default for MedevacEquipment {
    fn default() -> Self {
        Self::None
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteSecurity {
    NoEnemy,
    PossibleEnemy,
    EnemyInArea,
    ArmedEscortRequired,
    Custom(String),
}

impl Default for SiteSecurity {
    fn default() -> Self {
        Self::NoEnemy
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HlzMarking {
    Panels,
    PyrotechnicSignal,
    Smoke(String),
    None,
    Custom(String),
}

impl Default for HlzMarking {
    fn default() -> Self {
        Self::None
    }
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
        let lower = s.trim().to_lowercase();
        if lower.contains("panel") {
            Self::Panels
        } else if lower.contains("pyro") {
            Self::PyrotechnicSignal
        } else if lower.contains("smoke") {
            Self::Smoke(s.trim().to_string())
        } else if lower == "none" || lower.is_empty() {
            Self::None
        } else {
            Self::Custom(s.trim().to_string())
        }
    }
}

/// Line 8: Patient nationality and status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatientNationality {
    UsMilitary,
    UsCivilian,
    NonUsMilitary,
    NonUsCivilian,
    Epw,
    Child,
    Custom(String),
}

impl Default for PatientNationality {
    fn default() -> Self {
        Self::UsMilitary
    }
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
        if upper.contains("US MIL") {
            Self::UsMilitary
        } else if upper.contains("US CIV") {
            Self::UsCivilian
        } else if upper.contains("NON-US MIL") {
            Self::NonUsMilitary
        } else if upper.contains("NON-US CIV") {
            Self::NonUsCivilian
        } else if upper.contains("EPW") || upper.contains("DETAIN") {
            Self::Epw
        } else if upper.contains("CHILD") {
            Self::Child
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
