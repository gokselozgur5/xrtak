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
