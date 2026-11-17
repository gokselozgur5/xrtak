//! TAK GeoChatPacket XML serializer and parser.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    pub conversation_id: Option<String>,
    pub parent_message_id: Option<String>,
    pub recipients: Vec<String>,
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
            conversation_id: None,
            parent_message_id: None,
            recipients: Vec::new(),
        }
    }
    pub fn with_coordinates(mut self, lat: f64, lon: f64, hae: f64) -> Self {
        self.lat = lat;
        self.lon = lon;
        self.hae = hae;
        self
    }

    pub fn with_parent_group(mut self, group: impl Into<String>) -> Self {
        self.parent_group = group.into();
        self
    }

    pub fn with_conversation(mut self, conv_id: impl Into<String>, parent_id: Option<String>) -> Self {
        self.conversation_id = Some(conv_id.into());
        self.parent_message_id = parent_id;
        self
    }

    pub fn add_recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipients.push(recipient.into());
        self
    }

    pub fn with_recipients<I, S>(mut self, recipients: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for r in recipients {
            self.recipients.push(r.into());
        }
        self
    }
    /// Serializes packet to ATAK CoT XML string.
    pub fn to_cot_xml(&self) -> String {
        let msg = escape_xml(&self.message_text);
        let cs = escape_xml(&self.sender_callsign);
        let room = escape_xml(&self.chatroom);
        let uid = escape_xml(&self.sender_uid);
        let mid = escape_xml(&self.message_id);
        let e_uid = format!("GeoChat.{}.{}.{}", uid, room, mid);
        let t = &self.timestamp_iso;
        let mut out = String::new();
        out.push_str(&format!(
            "<event version=\"2.0\" uid=\"{e_uid}\" type=\"b-t-f\" how=\"h-g-i-g-o\" time=\"{t}\" start=\"{t}\" stale=\"{t}\">"
        ));
        out.push_str(&format!(
            "<point lat=\"{:.6}\" lon=\"{:.6}\" hae=\"{:.1}\" ce=\"9999999.0\" le=\"9999999.0\"/>",
            self.lat, self.lon, self.hae
        ));
        out.push_str("<detail>");
        let mut chat_tag = format!(
            "<__chat parent=\"{grp}\" groupOwner=\"false\" messageId=\"{mid}\" chatroom=\"{room}\" id=\"{room}\" senderCallsign=\"{cs}\"",
            grp = self.parent_group
        );
        if let Some(ref cid) = self.conversation_id {
            chat_tag.push_str(&format!(" conversationId=\"{cid}\""));
        }
        if let Some(ref pmid) = self.parent_message_id {
            chat_tag.push_str(&format!(" parentMessageId=\"{pmid}\""));
        }
        chat_tag.push('>');
        out.push_str(&chat_tag);
        let mut grp_tag = format!("<chatgrp id=\"{room}\" uid0=\"{uid}\"");
        for (i, r) in self.recipients.iter().enumerate() {
            grp_tag.push_str(&format!(" uid{}=\"{r}\"", i + 1));
        }
        grp_tag.push_str("/>");
        out.push_str(&grp_tag);
        out.push_str("</__chat>");
        out.push_str(&format!("<link uid=\"{uid}\" type=\"a-f-G-U-C\" relation=\"p-p\"/>"));
        out.push_str(&format!("<remarks source=\"BAO.F.ATAK.{uid}\" to=\"{room}\" time=\"{t}\">{msg}</remarks>"));
        out.push_str("</detail></event>");
        out
    }
    /// Parses an ATAK CoT XML message into a GeoChatPacket.
    pub fn from_cot_xml(xml: &str) -> Result<Self, GeoChatError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut in_remarks = false;
        let mut found_event = false;
        let mut sender_uid = String::new();
        let mut sender_callsign = String::new();
        let mut chatroom = String::new();
        let mut message_id = String::new();
        let mut message_text = String::new();
        let mut timestamp_iso = String::new();
        let mut parent_group = "RootContactGroup".to_string();
        let mut conversation_id = None;
        let mut parent_message_id = None;
        let mut recipients = Vec::new();
        let (mut lat, mut lon, mut hae) = (0.0, 0.0, 9999999.0);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"event" => {
                        found_event = true;
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"time" {
                                timestamp_iso = String::from_utf8_lossy(&a.value).into_owned();
                            }
                        }
                    }
                    b"point" => {
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"lat" => lat = parse_f64(&a.value).unwrap_or(0.0),
                                b"lon" => lon = parse_f64(&a.value).unwrap_or(0.0),
                                b"hae" => hae = parse_f64(&a.value).unwrap_or(9999999.0),
                                _ => {}
                            }
                        }
                    }
                    b"__chat" => {
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"senderCallsign" => sender_callsign = String::from_utf8_lossy(&a.value).into_owned(),
                                b"chatroom" => chatroom = String::from_utf8_lossy(&a.value).into_owned(),
                                b"messageId" => message_id = String::from_utf8_lossy(&a.value).into_owned(),
                                b"parent" => parent_group = String::from_utf8_lossy(&a.value).into_owned(),
                                b"conversationId" => conversation_id = Some(String::from_utf8_lossy(&a.value).into_owned()),
                                b"parentMessageId" => parent_message_id = Some(String::from_utf8_lossy(&a.value).into_owned()),
                                _ => {}
                            }
                        }
                    }
                    b"chatgrp" => {
                        for a in e.attributes().flatten() {
                            let k = a.key.as_ref();
                            if k.starts_with(b"uid") && k != b"uid0" {
                                let v = String::from_utf8_lossy(&a.value).into_owned();
                                if !v.is_empty() && !recipients.contains(&v) {
                                    recipients.push(v);
                                }
                            }
                        }
                    }
                    b"link" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"uid" && sender_uid.is_empty() {
                                sender_uid = String::from_utf8_lossy(&a.value).into_owned();
                            }
                        }
                    }
                    b"remarks" => in_remarks = true,
                    _ => {}
                },
                Ok(Event::Text(ref e)) => {
                    if in_remarks {
                        let raw = String::from_utf8_lossy(e.as_ref());
                        message_text.push_str(&unescape_xml(&raw));
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"remarks" { in_remarks = false; }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(GeoChatError::InvalidXml(e.to_string())),
                _ => {}
            }
        }
        if !found_event { return Err(GeoChatError::MissingEventTag); }
        if message_id.is_empty() { return Err(GeoChatError::MissingMessageId); }
        if sender_uid.is_empty() { return Err(GeoChatError::MissingSenderUid); }
        Ok(Self {
            sender_uid, sender_callsign, chatroom, message_id,
            message_text, timestamp_iso, lat, lon, hae, parent_group,
            conversation_id, parent_message_id, recipients,
        })
    }
}

/// Builder for constructing tactical GeoChatPacket instances.
#[derive(Debug, Clone, Default)]
pub struct GeoChatBuilder {
    sender_uid: Option<String>,
    sender_callsign: Option<String>,
    chatroom: Option<String>,
    message_id: Option<String>,
    message_text: Option<String>,
    timestamp_iso: Option<String>,
    lat: f64,
    lon: f64,
    hae: f64,
    parent_group: String,
    conversation_id: Option<String>,
    parent_message_id: Option<String>,
    recipients: Vec<String>,
}

impl GeoChatBuilder {
    pub fn new() -> Self {
        Self {
            parent_group: "RootContactGroup".to_string(),
            hae: 9999999.0,
            ..Default::default()
        }
    }

    pub fn sender(mut self, uid: impl Into<String>, callsign: impl Into<String>) -> Self {
        self.sender_uid = Some(uid.into());
        self.sender_callsign = Some(callsign.into());
        self
    }
    pub fn chatroom(mut self, chatroom: impl Into<String>) -> Self {
        self.chatroom = Some(chatroom.into());
        self
    }

    pub fn message(mut self, id: impl Into<String>, text: impl Into<String>) -> Self {
        self.message_id = Some(id.into());
        self.message_text = Some(text.into());
        self
    }
    pub fn timestamp(mut self, iso: impl Into<String>) -> Self {
        self.timestamp_iso = Some(iso.into());
        self
    }

    pub fn coordinates(mut self, lat: f64, lon: f64, hae: f64) -> Self {
        self.lat = lat;
        self.lon = lon;
        self.hae = hae;
        self
    }
    pub fn parent_group(mut self, group: impl Into<String>) -> Self {
        self.parent_group = group.into();
        self
    }

    pub fn conversation(mut self, conv_id: impl Into<String>, parent_id: Option<String>) -> Self {
        self.conversation_id = Some(conv_id.into());
        self.parent_message_id = parent_id;
        self
    }
    pub fn add_recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipients.push(recipient.into());
        self
    }

    pub fn recipients<I, S>(mut self, recips: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for r in recips {
            self.recipients.push(r.into());
        }
        self
    }
    pub fn build(self) -> Result<GeoChatPacket, GeoChatError> {
        let sender_uid = self.sender_uid.ok_or(GeoChatError::MissingSenderUid)?;
        let sender_callsign = self.sender_callsign.unwrap_or_else(|| "UNKNOWN".to_string());
        let chatroom = self.chatroom.unwrap_or_else(|| "All Chat Rooms".to_string());
        let message_id = self.message_id.ok_or(GeoChatError::MissingMessageId)?;
        let message_text = self.message_text.unwrap_or_default();
        let timestamp_iso = self.timestamp_iso.unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        Ok(GeoChatPacket {
            sender_uid,
            sender_callsign,
            chatroom,
            message_id,
            message_text,
            timestamp_iso,
            lat: self.lat,
            lon: self.lon,
            hae: self.hae,
            parent_group: self.parent_group,
            conversation_id: self.conversation_id,
            parent_message_id: self.parent_message_id,
            recipients: self.recipients,
        })
    }
}

/// A conversation thread aggregating related GeoChat messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationThread {
    pub thread_id: String,
    pub root_message_id: Option<String>,
    pub messages: Vec<GeoChatPacket>,
    pub participants: Vec<String>,
}

impl ConversationThread {
    pub fn new(thread_id: impl Into<String>) -> Self {
        Self {
            thread_id: thread_id.into(),
            root_message_id: None,
            messages: Vec::new(),
            participants: Vec::new(),
        }
    }
    pub fn add_message(&mut self, pkt: GeoChatPacket) {
        if self.messages.is_empty() && self.root_message_id.is_none() {
            self.root_message_id = Some(pkt.message_id.clone());
        }
        if !self.participants.contains(&pkt.sender_uid) {
            self.participants.push(pkt.sender_uid.clone());
        }
        for r in &pkt.recipients {
            if !self.participants.contains(r) {
                self.participants.push(r.clone());
            }
        }
        self.messages.push(pkt);
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn last_message(&self) -> Option<&GeoChatPacket> {
        self.messages.last()
    }
}

/// Manages and groups incoming GeoChat packets into conversation threads.
#[derive(Debug, Clone, Default)]
pub struct GeoChatThreader {
    threads: BTreeMap<String, ConversationThread>,
    message_to_thread: HashMap<String, String>,
}

impl GeoChatThreader {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn ingest(&mut self, pkt: GeoChatPacket) -> String {
        let thread_id = if let Some(ref cid) = pkt.conversation_id {
            cid.clone()
        } else if let Some(ref pmid) = pkt.parent_message_id {
            self.message_to_thread.get(pmid).cloned().unwrap_or_else(|| pmid.clone())
        } else {
            pkt.chatroom.clone()
        };

        self.message_to_thread.insert(pkt.message_id.clone(), thread_id.clone());
        let thread = self.threads.entry(thread_id.clone()).or_insert_with(|| {
            ConversationThread::new(&thread_id)
        });
        thread.add_message(pkt);
        thread_id
    }
    pub fn thread(&self, thread_id: &str) -> Option<&ConversationThread> {
        self.threads.get(thread_id)
    }

    pub fn thread_count(&self) -> usize {
        self.threads.len()
    }

    pub fn threads(&self) -> impl Iterator<Item = &ConversationThread> {
        self.threads.values()
    }

    pub fn get_replies(&self, parent_message_id: &str) -> Vec<&GeoChatPacket> {
        self.threads
            .values()
            .flat_map(|t| t.messages.iter())
            .filter(|m| m.parent_message_id.as_deref() == Some(parent_message_id))
            .collect()
    }

    pub fn all_participants(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for t in self.threads.values() {
            for p in &t.participants {
                set.insert(p.clone());
            }
        }
        set.into_iter().collect()
    }
}

fn parse_f64(bytes: &[u8]) -> Option<f64> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geochat_roundtrip() {
        let pkt = GeoChatPacket::new(
            "UID-VIPER-1",
            "VIPER-1",
            "Alpha-Squad",
            "MSG-9901",
            "Target location identified & verified.",
            "2026-11-16T12:00:00Z",
        ).with_coordinates(34.123456, -118.654321, 150.0);

        let xml = pkt.to_cot_xml();
        let parsed = GeoChatPacket::from_cot_xml(&xml).expect("roundtrip parse failed");
        assert_eq!(parsed.sender_uid, "UID-VIPER-1");
        assert_eq!(parsed.sender_callsign, "VIPER-1");
        assert_eq!(parsed.chatroom, "Alpha-Squad");
        assert_eq!(parsed.message_text, "Target location identified & verified.");
        assert_eq!(parsed.lat, 34.123456);
    }
    #[test]
    fn test_xml_escaping_in_message() {
        let pkt = GeoChatPacket::new(
            "UID-1", "LEAD", "HQ", "M-1",
            "Danger: <Enemy> & Alert!",
            "2026-11-16T12:00:00Z",
        );
        let xml = pkt.to_cot_xml();
        assert!(xml.contains("&lt;Enemy&gt;"));
        assert!(xml.contains("&amp;"));
        let parsed = GeoChatPacket::from_cot_xml(&xml).unwrap();
        assert_eq!(parsed.message_text, "Danger: <Enemy> & Alert!");
    }

    #[test]
    fn test_missing_fields_validation() {
        let bad_xml = "<event version=\"2.0\"></event>";
        let err = GeoChatPacket::from_cot_xml(bad_xml).unwrap_err();
        assert_eq!(err, GeoChatError::MissingMessageId);
    }
}
