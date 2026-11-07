// crates/xrtak-core/src/battery.rs
//! UAV Wind-Corrected Bingo Battery & Asymmetric RTH Polygon Engine (ICD-26, REQ-BAT-01..02).

use serde::{Deserialize, Serialize};

/// Ambient wind vector in local ENU frame.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindVector {
    /// Wind blowing towards East (m/s)
    pub east_mps: f64,
    /// Wind blowing towards North (m/s)
    pub north_mps: f64,
}

impl Default for WindVector {
    fn default() -> Self {
        Self { east_mps: 0.0, north_mps: 0.0 }
    }
}
