use serde::{Deserialize, Serialize};

/// Distance attenuation model for radio / audio emitters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttenuationModel {
    InverseDistance,
    Linear,
    Exponential,
    None,
}

impl Default for AttenuationModel {
    fn default() -> Self {
        Self::InverseDistance
    }
}

/// Parameters controlling spatial distance attenuation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttenuationParams {
    pub min_distance_m: f64,
    pub max_distance_m: f64,
    pub rolloff_factor: f64,
    pub model: AttenuationModel,
}
