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

impl AudioEmitter {
    pub fn new(id: impl Into<String>, position: AudioVec3) -> Self {
        Self {
            id: id.into(),
            position,
            volume: 1.0,
            attenuation: AttenuationParams::default(),
        }
    }

    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }

    pub fn with_attenuation(mut self, attenuation: AttenuationParams) -> Self {
        self.attenuation = attenuation;
        self
    }
}
