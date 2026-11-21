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
        let lower = err_str.to_ascii_lowercase();
        if lower.starts_with("xml parse error:") {
            CotHarnessError::InvalidXml(err_str)
        } else if lower.contains("missing cot event uid") || lower.contains("missing uid") {
            CotHarnessError::MissingUid
        } else if lower.contains("missing point") {
            CotHarnessError::MissingPoint
        } else if lower.contains("must be in")
            || lower.contains("range")
            || lower.contains("coordinate")
        {
            CotHarnessError::CoordinateRange(err_str)
        } else {
            CotHarnessError::InvalidXml(err_str)
        }
    }
}

impl CotParseHarness {
    pub fn parse_raw(&mut self, raw: &RawCotMessage) -> Result<ParsedCotMessage, CotHarnessError> {
        self.parse_raw_with_provider(raw, &ProceduralReliefModel)
    }
}

impl CotParseHarness {
    fn record_affiliation(&mut self, aff: Affiliation) {
        match aff {
            Affiliation::Friendly => self.metrics.friendly_count += 1,
            Affiliation::Hostile => self.metrics.hostile_count += 1,
            Affiliation::Neutral => self.metrics.neutral_count += 1,
            Affiliation::Unknown => self.metrics.unknown_count += 1,
        }
        self.metrics.total_success += 1;
    }
}

impl CotParseHarness {
    pub fn parse_raw_with_provider(
        &mut self,
        raw: &RawCotMessage,
        provider: &dyn ElevationProvider,
    ) -> Result<ParsedCotMessage, CotHarnessError> {
        self.metrics.total_processed += 1;
        self.metrics.total_bytes_ingested += raw.payload.len();
        let start = std::time::Instant::now();
        let res =
            parse_cot_xml_with_provider(&raw.payload, &self.origin, provider, raw.received_unix_ms);
        let elapsed = start.elapsed().as_micros() as u64;
        let entity = res.map_err(|e| {
            self.metrics.total_failed += 1;
            Self::map_error(e)
        })?;
        self.record_affiliation(entity.affiliation);
        Ok(ParsedCotMessage {
            entity,
            raw_bytes_len: raw.payload.len(),
            parse_duration_micros: elapsed,
        })
    }
}

impl CotParseHarness {
    pub fn parse_batch(
        &mut self,
        messages: &[RawCotMessage],
    ) -> Vec<Result<ParsedCotMessage, CotHarnessError>> {
        messages.iter().map(|msg| self.parse_raw(msg)).collect()
    }
}
