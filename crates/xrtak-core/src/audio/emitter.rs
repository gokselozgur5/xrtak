use super::attenuation::AttenuationParams;
use super::types::AudioVec3;
use serde::{Deserialize, Serialize};

/// 3D audio emitter representing a radio call or tactical audio beacon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEmitter {
    pub id: String,
    pub position: AudioVec3,
    pub volume: f64,
    pub attenuation: AttenuationParams,
}
