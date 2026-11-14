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
