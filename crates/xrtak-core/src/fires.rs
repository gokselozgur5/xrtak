// crates/xrtak-core/src/fires.rs
//! 3D Ballistic Trajectory, Maximum Ordinate & Crest Clearance Engine (ICD-25, REQ-FIR-01..02).

use serde::{Deserialize, Serialize};
use crate::geo::ElevationProvider;

/// Fire support munition / trajectory profile.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MunitionType {
    /// 155mm Howitzer / Artillery shell (high angle or low angle)
    Artillery155mm,
    /// 120mm Heavy Mortar (high angle plunging)
    Mortar120mm,
    /// Loitering Munition / Kamikaze Drone (glide profile)
    LoiteringMunition,
    /// Direct Rocket / Guided Rocket (MLRS)
    TacticalRocket,
}

impl MunitionType {
    pub fn nominal_velocity_mps(&self) -> f64 {
        match self {
            Self::Artillery155mm => 580.0,
            Self::Mortar120mm => 240.0,
            Self::LoiteringMunition => 65.0,
            Self::TacticalRocket => 820.0,
        }
    }
}
