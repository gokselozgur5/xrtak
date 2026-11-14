use crate::domain::Affiliation;
use crate::geo::{ElevationProvider, ProceduralReliefModel};
use crate::{parse_cot_xml_with_provider, GeoOrigin, TacticalEntity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CotHarnessError {
    InvalidXml(String),
    MissingPoint,
    MissingUid,
    CoordinateRange(String),
}

impl std::fmt::Display for CotHarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            Self::MissingPoint => write!(f, "Missing point element"),
            Self::MissingUid => write!(f, "Missing uid attribute"),
            Self::CoordinateRange(e) => write!(f, "Coordinate out of range: {e}"),
        }
    }
}

impl std::error::Error for CotHarnessError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCotMessage {
    pub payload: String,
    pub received_unix_ms: u64,
}

impl RawCotMessage {
    pub fn new(payload: impl Into<String>, received_unix_ms: u64) -> Self {
        Self {
            payload: payload.into(),
            received_unix_ms,
        }
    }
}
