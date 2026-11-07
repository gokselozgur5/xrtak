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

/// Single sample along 3D ballistic trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint3d {
    pub time_sec: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
    pub terrain_elev_m: f64,
    pub clearance_m: f64,
}

/// Ballistic fire mission solution and terrain crest analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallisticSolution {
    pub munition: MunitionType,
    pub ground_range_m: f64,
    pub time_of_flight_sec: f64,
    pub max_ordinate_m: f64,
    pub min_crest_clearance_m: f64,
    pub is_crest_cleared: bool,
    pub trajectory: Vec<TrajectoryPoint3d>,
}
