use serde::{Deserialize, Serialize};

/// Distance attenuation model for radio / audio emitters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttenuationModel {
    InverseDistance,
    Linear,
    Exponential,
    None,
}
