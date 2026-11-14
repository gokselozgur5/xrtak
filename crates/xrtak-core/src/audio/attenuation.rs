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

impl Default for AttenuationParams {
    fn default() -> Self {
        Self {
            min_distance_m: 1.0,
            max_distance_m: 500.0,
            rolloff_factor: 1.0,
            model: AttenuationModel::default(),
        }
    }
}

pub fn compute_distance_attenuation(distance_m: f64, params: &AttenuationParams) -> f64 {
    if params.model == AttenuationModel::None || distance_m <= params.min_distance_m {
        return 1.0;
    }
    if distance_m >= params.max_distance_m {
        return 0.0;
    }
    match params.model {
        AttenuationModel::None => 1.0,
        AttenuationModel::InverseDistance => {
            (params.min_distance_m
                / (params.min_distance_m + params.rolloff_factor * (distance_m - params.min_distance_m)))
                .clamp(0.0, 1.0)
        }
        AttenuationModel::Linear => {
            let span = params.max_distance_m - params.min_distance_m;
            if span <= 0.0 {
                return 0.0;
            }
            (1.0 - params.rolloff_factor * (distance_m - params.min_distance_m) / span).clamp(0.0, 1.0)
        }
        AttenuationModel::Exponential => {
            (distance_m / params.min_distance_m).max(1.0).powf(-params.rolloff_factor).clamp(0.0, 1.0)
        }
    }
}
