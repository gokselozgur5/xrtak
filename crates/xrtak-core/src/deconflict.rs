//! 4D Airspace Deconfliction, Pairwise CPA & ROZ Collision Avoidance Engine.
//!
//! Complies with REQ-DEC-01..02 and ICD-21 for multi-UAV spatial separation.

use serde::{Deserialize, Serialize};

/// Airspace conflict collision risk classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpaRiskLevel {
    /// Severe impending collision risk (t_cpa <= 60s, d_h < 300m, d_v < 100m).
    Critical,
    /// Proximity advisory warning (t_cpa <= 180s, d_h < 1000m, d_v < 250m).
    Advisory,
    /// Safe separation / diverging trajectories.
    Clear,
}

/// Pairwise CPA geometry between two moving tactical air tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirspaceConflictReport {
    pub entity_a: String,
    pub entity_b: String,
    pub t_cpa_sec: f64,
    pub horiz_separation_m: f64,
    pub vert_separation_m: f64,
    pub risk_level: CpaRiskLevel,
    pub pos_a_at_cpa: (f64, f64, f64),
    pub pos_b_at_cpa: (f64, f64, f64),
}

/// Aggregate deconfliction analysis summary for theater airspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeconflictionResult {
    pub conflicts: Vec<AirspaceConflictReport>,
    pub max_risk: CpaRiskLevel,
    pub total_evaluated_pairs: usize,
}
