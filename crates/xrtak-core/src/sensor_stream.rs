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
