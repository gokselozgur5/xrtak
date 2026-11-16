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
