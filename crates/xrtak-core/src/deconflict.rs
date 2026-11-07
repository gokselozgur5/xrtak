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

/// Computes raw time to closest approach (t_cpa) and relative velocity vector.
pub fn compute_raw_t_cpa(
    pos_a: (f64, f64, f64),
    va: (f64, f64, f64),
    pos_b: (f64, f64, f64),
    vb: (f64, f64, f64),
) -> (f64, (f64, f64, f64)) {
    let rx = pos_a.0 - pos_b.0;
    let ry = pos_a.1 - pos_b.1;
    let rz = pos_a.2 - pos_b.2;
    let v_rel = (va.0 - vb.0, va.1 - vb.1, va.2 - vb.2);
    let v_sq = v_rel.0 * v_rel.0 + v_rel.1 * v_rel.1 + v_rel.2 * v_rel.2;
    let t_cpa = if v_sq > 1e-4 {
        -(rx * v_rel.0 + ry * v_rel.1 + rz * v_rel.2) / v_sq
    } else {
        0.0
    };
    (t_cpa, v_rel)
}

/// Calculates Closest Point of Approach (CPA) between two 3D moving entities.
pub fn compute_pairwise_cpa(
    uid_a: &str,
    pos_a: (f64, f64, f64),
    speed_a_mps: f64,
    course_a_deg: f64,
    uid_b: &str,
    pos_b: (f64, f64, f64),
    speed_b_mps: f64,
    course_b_deg: f64,
) -> AirspaceConflictReport {
    let rad_a = course_a_deg.to_radians();
    let rad_b = course_b_deg.to_radians();
    let va = (speed_a_mps * rad_a.sin(), speed_a_mps * rad_a.cos(), 0.0);
    let vb = (speed_b_mps * rad_b.sin(), speed_b_mps * rad_b.cos(), 0.0);

    let (t_cpa_raw, _) = compute_raw_t_cpa(pos_a, va, pos_b, vb);
    let t_eval = t_cpa_raw.max(0.0);
    let p_a = (pos_a.0 + va.0 * t_eval, pos_a.1 + va.1 * t_eval, pos_a.2 + va.2 * t_eval);
    let p_b = (pos_b.0 + vb.0 * t_eval, pos_b.1 + vb.1 * t_eval, pos_b.2 + vb.2 * t_eval);

    let dx = p_a.0 - p_b.0;
    let dy = p_a.1 - p_b.1;
    let d_horiz = (dx * dx + dy * dy).sqrt();
    let d_vert = (p_a.2 - p_b.2).abs();

    let risk = if t_cpa_raw >= 0.0 && t_cpa_raw <= 60.0 && d_horiz < 300.0 && d_vert < 100.0 {
        CpaRiskLevel::Critical
    } else if t_cpa_raw >= 0.0 && t_cpa_raw <= 180.0 && d_horiz < 1000.0 && d_vert < 250.0 {
        CpaRiskLevel::Advisory
    } else {
        CpaRiskLevel::Clear
    };

    AirspaceConflictReport {
        entity_a: uid_a.to_string(),
        entity_b: uid_b.to_string(),
        t_cpa_sec: (t_cpa_raw * 10.0).round() / 10.0,
        horiz_separation_m: (d_horiz * 10.0).round() / 10.0,
        vert_separation_m: (d_vert * 10.0).round() / 10.0,
        risk_level: risk,
        pos_a_at_cpa: p_a,
        pos_b_at_cpa: p_b,
    }
}
