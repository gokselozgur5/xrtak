//! TAK GeoChatPacket XML serializer and parser.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during GeoChat XML parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeoChatError {
    InvalidXml(String),
    MissingEventTag,
    MissingChatTag,
    MissingRemarks,
    MissingSenderUid,
    MissingMessageId,
}

impl fmt::Display for GeoChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {}", e),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingChatTag => write!(f, "Missing chat tag"),
            Self::MissingRemarks => write!(f, "Missing remarks"),
            Self::MissingSenderUid => write!(f, "Missing sender UID"),
            Self::MissingMessageId => write!(f, "Missing messageId"),
        }
    }
}
