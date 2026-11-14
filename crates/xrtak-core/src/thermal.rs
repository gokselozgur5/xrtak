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

impl Default for ThermalShaderConfig {
    fn default() -> Self {
        Self {
            palette: ThermalPalette::WhiteHot,
            min_temp_c: 0.0,
            max_temp_c: 50.0,
            gain: 1.0,
            noise_intensity: 0.02,
        }
    }
}

/// RGB color with float components in [0.0, 1.0].
