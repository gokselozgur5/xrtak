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

/// Cursor-on-Target identity affiliations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoTAffiliation {
    Friendly,
    Hostile,
    Neutral,
    Unknown,
    AssumedFriend,
    Suspect,
    Joker,
    Faker,
    Pending,
}

impl CoTAffiliation {
    /// Return the standard CoT 1-letter affiliation character.
    pub fn to_char(self) -> char {
        match self {
            Self::Friendly => 'f',
            Self::Hostile => 'h',
            Self::Neutral => 'n',
            Self::Unknown => 'u',
            Self::AssumedFriend => 'a',
            Self::Suspect => 's',
            Self::Joker => 'j',
            Self::Faker => 'k',
            Self::Pending => 'p',
        }
    }
    /// Parse CoTAffiliation from a 1-letter code character.
    pub fn from_char(c: char) -> Self {
        match c.to_ascii_lowercase() {
            'f' => Self::Friendly,
            'h' => Self::Hostile,
            'n' => Self::Neutral,
            'u' => Self::Unknown,
            'a' => Self::AssumedFriend,
            's' => Self::Suspect,
            'j' => Self::Joker,
            'k' => Self::Faker,
            'p' => Self::Pending,
            _ => Self::Unknown,
        }
    }
}

/// Standard tactical identity classification categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StandardIdentity {
    Friendly,
    Hostile,
    Neutral,
    Unknown,
}

impl CoTAffiliation {
    /// Map detailed CoT affiliation to high-level standard identity.
    pub fn to_standard_identity(self) -> StandardIdentity {
        match self {
            Self::Friendly | Self::AssumedFriend => StandardIdentity::Friendly,
            Self::Hostile | Self::Suspect | Self::Joker | Self::Faker => StandardIdentity::Hostile,
            Self::Neutral => StandardIdentity::Neutral,
            Self::Unknown | Self::Pending => StandardIdentity::Unknown,
        }
    }
}

/// 32-bit RGBA tactical color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TacticalColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl TacticalColor {
    /// Create a new RGBA tactical color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create an opaque tactical color from RGB components.
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}
