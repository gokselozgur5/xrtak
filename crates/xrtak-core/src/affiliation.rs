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

/// Tactical contact marker representation for visualization and tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactMarker {
    pub uid: String,
    pub callsign: String,
    pub cot_type: String,
    pub affiliation: CoTAffiliation,
    pub standard_identity: StandardIdentity,
    pub color: TacticalColor,
    pub shape: MarkerShapeGlyph,
    pub threat_level: TacticalThreatLevel,
    pub is_combatant: bool,
    pub is_custom_color: bool,
}

/// Aggregated statistics across tactical contact markers.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactStats {
    pub total: usize,
    pub friendly_count: usize,
    pub hostile_count: usize,
    pub neutral_count: usize,
    pub unknown_count: usize,
    pub suspect_count: usize,
    pub high_threat_count: usize,
}

/// Mapper engine for CoT contact affiliations and marker visual coding.
pub struct AffiliationMapper;

impl AffiliationMapper {
    /// Extract CoTAffiliation from a CoT type string.
    pub fn parse_affiliation(cot_type: &str) -> CoTAffiliation {
        let parts: Vec<&str> = cot_type.split('-').collect();
        if parts.len() < 2 || parts[0] != "a" {
            return CoTAffiliation::Unknown;
        }
        let aff_char = parts[1].chars().next().unwrap_or('u');
        CoTAffiliation::from_char(aff_char)
    }
}

impl AffiliationMapper {
    /// Map CoT affiliation to designated tactical color.
    pub fn map_color(affil: CoTAffiliation) -> TacticalColor {
        match affil {
            CoTAffiliation::Friendly => TacticalColor::FRIENDLY_BLUE,
            CoTAffiliation::AssumedFriend => TacticalColor::ASSUMED_FRIEND_CYAN,
            CoTAffiliation::Hostile | CoTAffiliation::Joker | CoTAffiliation::Faker => {
                TacticalColor::HOSTILE_RED
            }
            CoTAffiliation::Suspect => TacticalColor::SUSPECT_ORANGE,
            CoTAffiliation::Neutral => TacticalColor::NEUTRAL_GREEN,
            CoTAffiliation::Unknown | CoTAffiliation::Pending => TacticalColor::UNKNOWN_YELLOW,
        }
    }
}

impl AffiliationMapper {
    /// Map CoT affiliation to 3D marker geometric glyph shape.
    pub fn map_shape(affil: CoTAffiliation) -> MarkerShapeGlyph {
        match affil {
            CoTAffiliation::Friendly | CoTAffiliation::AssumedFriend => MarkerShapeGlyph::Rectangle,
            CoTAffiliation::Hostile | CoTAffiliation::Joker | CoTAffiliation::Faker => {
                MarkerShapeGlyph::Diamond
            }
            CoTAffiliation::Suspect => MarkerShapeGlyph::DashedDiamond,
            CoTAffiliation::Neutral => MarkerShapeGlyph::Square,
            CoTAffiliation::Unknown | CoTAffiliation::Pending => MarkerShapeGlyph::Quatrefoil,
        }
    }
}

impl AffiliationMapper {
    /// Map CoT affiliation to tactical threat severity level.
    pub fn map_threat(affil: CoTAffiliation) -> TacticalThreatLevel {
        match affil {
            CoTAffiliation::Hostile | CoTAffiliation::Joker | CoTAffiliation::Faker => {
                TacticalThreatLevel::High
            }
            CoTAffiliation::Suspect => TacticalThreatLevel::Medium,
            CoTAffiliation::Unknown | CoTAffiliation::Pending => TacticalThreatLevel::Low,
            CoTAffiliation::Friendly | CoTAffiliation::AssumedFriend | CoTAffiliation::Neutral => {
                TacticalThreatLevel::None
            }
        }
    }
}

impl AffiliationMapper {
    /// Construct a ContactMarker with optional custom ATAK ARGB color override.
    pub fn create_marker(
        uid: &str,
        callsign: &str,
        cot_type: &str,
        custom_argb: Option<i32>,
    ) -> ContactMarker {
        let affiliation = Self::parse_affiliation(cot_type);
        let standard_identity = affiliation.to_standard_identity();
        let (color, is_custom_color) = match custom_argb {
            Some(argb) => (TacticalColor::from_argb_i32(argb), true),
            None => (Self::map_color(affiliation), false),
        };
        let shape = Self::map_shape(affiliation);
        let threat_level = Self::map_threat(affiliation);
        let is_combatant = threat_level == TacticalThreatLevel::High;

        ContactMarker {
            uid: uid.to_string(),
            callsign: callsign.to_string(),
            cot_type: cot_type.to_string(),
            affiliation,
            standard_identity,
            color,
            shape,
            threat_level,
            is_combatant,
            is_custom_color,
        }
    }
}

impl AffiliationMapper {
    /// Aggregate statistical distribution across multiple contact markers.
    pub fn evaluate_contacts(markers: &[ContactMarker]) -> ContactStats {
        let mut stats = ContactStats {
            total: markers.len(),
            ..Default::default()
        };
        for m in markers {
            match m.affiliation {
                CoTAffiliation::Friendly | CoTAffiliation::AssumedFriend => {
                    stats.friendly_count += 1;
                }
                CoTAffiliation::Hostile | CoTAffiliation::Joker | CoTAffiliation::Faker => {
                    stats.hostile_count += 1;
                }
                CoTAffiliation::Neutral => stats.neutral_count += 1,
                CoTAffiliation::Unknown | CoTAffiliation::Pending => stats.unknown_count += 1,
                CoTAffiliation::Suspect => stats.suspect_count += 1,
            }
            if m.threat_level == TacticalThreatLevel::High {
                stats.high_threat_count += 1;
            }
        }
        stats
    }
}

impl AffiliationMapper {
    /// Format ATAK compliant `<color argb="..."/>` XML element.
    pub fn to_cot_color_xml_element(marker: &ContactMarker) -> String {
        format!("<color argb=\"{}\"/>", marker.color.to_atak_argb_i32())
    }

    /// Construct a CoT 2525 type string from affiliation and dimension suffix.
    pub fn to_cot_type(affil: CoTAffiliation, dimension_suffix: &str) -> String {
        format!("a-{}-{}", affil.to_char(), dimension_suffix)
    }
}

impl AffiliationMapper {
    /// Parse a CoT XML snippet into a populated ContactMarker.
    pub fn parse_cot_xml(xml: &str) -> Result<ContactMarker, AffiliationError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut uid = String::new();
        let mut cot_type = String::new();
        let mut callsign = String::new();
        let mut custom_argb: Option<i32> = None;
        let mut found_event = false;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                    b"event" => {
                        found_event = true;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"uid" {
                                uid = String::from_utf8_lossy(&attr.value).to_string();
                            } else if attr.key.as_ref() == b"type" {
                                cot_type = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    b"contact" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"callsign" {
                                callsign = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
