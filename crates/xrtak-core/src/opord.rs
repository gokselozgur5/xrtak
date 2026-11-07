//! 3D Tactical KML Scene Exporter & NATO 5-Paragraph OPORD Generator.
//!
//! Complies with REQ-OPO-01..02 and ICD-22 for operational order synthesis.

use serde::{Deserialize, Serialize};

/// Lightweight tactical entity snapshot for operational order summaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpordUnitSummary {
    pub callsign: String,
    pub cot_type: String,
    pub affiliation: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
}

/// Comprehensive tactical scene export bundle containing 3D KML and NATO OPORD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpordExportBundle {
    pub opord_markdown: String,
    pub kml_scene: String,
    pub unit_count: usize,
    pub zone_count: usize,
    pub dome_count: usize,
    pub generated_utc: String,
}
