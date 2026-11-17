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

impl TacticalColor {
    /// Convert color to normalized [0.0, 1.0] RGBA float tuple.
    pub fn to_rgba_f32(self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }

    /// Format color as an 8-character `#RRGGBBAA` hex string.
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

impl TacticalColor {
    /// Convert color to unsigned 32-bit ARGB representation.
    pub fn to_argb_u32(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }

    /// Convert color to signed 32-bit integer ARGB used by ATAK XML.
    pub fn to_atak_argb_i32(self) -> i32 {
        self.to_argb_u32() as i32
    }
}

impl TacticalColor {
    /// Parse color from signed 32-bit integer ARGB used by ATAK XML.
    pub fn from_argb_i32(val: i32) -> Self {
        let u = val as u32;
        let a = ((u >> 24) & 0xFF) as u8;
        let r = ((u >> 16) & 0xFF) as u8;
        let g = ((u >> 8) & 0xFF) as u8;
        let b = (u & 0xFF) as u8;
        Self { r, g, b, a }
    }
}

impl TacticalColor {
    /// Standard military friendly blue color.
    pub const FRIENDLY_BLUE: Self = Self::new(0, 165, 255, 255);
    /// Standard military hostile red color.
    pub const HOSTILE_RED: Self = Self::new(255, 38, 0, 255);
    /// Standard military neutral green color.
    pub const NEUTRAL_GREEN: Self = Self::new(0, 200, 83, 255);
    /// Standard military unknown yellow color.
    pub const UNKNOWN_YELLOW: Self = Self::new(255, 214, 0, 255);
    /// Suspect orange color.
    pub const SUSPECT_ORANGE: Self = Self::new(255, 145, 0, 255);
    /// Assumed friend cyan color.
    pub const ASSUMED_FRIEND_CYAN: Self = Self::new(0, 229, 255, 255);
}

/// Tactical contact marker geometric shape glyph type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarkerShapeGlyph {
    Rectangle,
    Circle,
    Diamond,
    Square,
    Quatrefoil,
    DashedDiamond,
}

/// Tactical threat severity assessment level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TacticalThreatLevel {
    None,
    Low,
    Medium,
    High,
}
