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
