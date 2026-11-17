//! CoT contact marker color coding and identity affiliation mapper.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during CoT affiliation mapping and parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AffiliationError {
    InvalidXml(String),
    MissingEventTag,
    MissingTypeAttribute,
    InvalidColorHex(String),
}

impl fmt::Display for AffiliationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingTypeAttribute => write!(f, "Missing type attribute"),
            Self::InvalidColorHex(h) => write!(f, "Invalid color hex: {h}"),
        }
    }
}

impl std::error::Error for AffiliationError {}
