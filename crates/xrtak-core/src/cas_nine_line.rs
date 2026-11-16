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
