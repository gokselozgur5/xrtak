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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCotMessage {
    pub entity: TacticalEntity,
    pub raw_bytes_len: usize,
    pub parse_duration_micros: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CotParseMetrics {
    pub total_processed: usize,
    pub total_success: usize,
    pub total_failed: usize,
    pub total_bytes_ingested: usize,
    pub friendly_count: usize,
    pub hostile_count: usize,
    pub neutral_count: usize,
    pub unknown_count: usize,
}

#[derive(Debug, Clone)]
pub struct CotParseHarness {
    pub origin: GeoOrigin,
    pub metrics: CotParseMetrics,
}

impl Default for CotParseHarness {
    fn default() -> Self {
        Self::new(GeoOrigin::default())
    }
}

impl CotParseHarness {
    pub fn new(origin: GeoOrigin) -> Self {
        Self {
            origin,
            metrics: CotParseMetrics::default(),
        }
    }

    pub fn reset_metrics(&mut self) {
        self.metrics = CotParseMetrics::default();
    }
}

impl CotParseHarness {
    fn map_error(err_str: String) -> CotHarnessError {
        if err_str.contains("missing point") || err_str.contains("point") {
            CotHarnessError::MissingPoint
        } else if err_str.contains("missing uid") {
            CotHarnessError::MissingUid
        } else if err_str.contains("range") || err_str.contains("coordinate") {
            CotHarnessError::CoordinateRange(err_str)
        } else {
            CotHarnessError::InvalidXml(err_str)
        }
    }
}
