use serde::{Deserialize, Serialize};

/// Standard thermal false-color palettes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalPalette {
    WhiteHot,
    BlackHot,
    Ironbow,
    Rainbow,
    GreenHot,
}

/// Thermal shader configuration parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermalShaderConfig {
    pub palette: ThermalPalette,
    pub min_temp_c: f32,
    pub max_temp_c: f32,
    pub gain: f32,
    pub noise_intensity: f32,
}

