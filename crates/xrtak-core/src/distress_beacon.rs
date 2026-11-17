//! CoT emergency beacon and 911 distress alert generator.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors during emergency beacon parsing and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistressBeaconError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingEmergencyTag,
    InvalidCoordinate(String),
}

