//! TAK sensor video stream descriptor XML generator and parser.
//!
//! Provides Cursor-on-Target (CoT) `b-i-v` XML event generation and
//! TAK video connection `<feed>` descriptor XML serialization.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during sensor video stream descriptor XML operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorStreamError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingDetailTag,
    MissingVideoTag,
    MissingFeedTag,
    MissingAttribute(&'static str),
}

impl fmt::Display for SensorStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {}", e),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingDetailTag => write!(f, "Missing detail tag"),
            Self::MissingVideoTag => write!(f, "Missing video stream tag"),
            Self::MissingFeedTag => write!(f, "Missing feed tag"),
            Self::MissingAttribute(a) => write!(f, "Missing attribute: {}", a),
        }
    }
}

impl std::error::Error for SensorStreamError {}
