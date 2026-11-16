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

impl std::error::Error for GeoChatError {}

/// Tactical instant message GeoChat packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoChatPacket {
    pub sender_uid: String,
    pub sender_callsign: String,
    pub chatroom: String,
    pub message_id: String,
    pub message_text: String,
    pub timestamp_iso: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub parent_group: String,
}

impl GeoChatPacket {
    pub fn new(
        sender_uid: impl Into<String>,
        sender_callsign: impl Into<String>,
        chatroom: impl Into<String>,
        message_id: impl Into<String>,
        message_text: impl Into<String>,
        timestamp_iso: impl Into<String>,
    ) -> Self {
        Self {
            sender_uid: sender_uid.into(),
            sender_callsign: sender_callsign.into(),
            chatroom: chatroom.into(),
            message_id: message_id.into(),
            message_text: message_text.into(),
            timestamp_iso: timestamp_iso.into(),
            lat: 0.0,
            lon: 0.0,
            hae: 9999999.0,
            parent_group: "RootContactGroup".to_string(),
        }
    }
