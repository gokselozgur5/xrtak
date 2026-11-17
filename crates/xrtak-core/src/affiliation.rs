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
