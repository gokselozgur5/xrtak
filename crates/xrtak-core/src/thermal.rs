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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorRgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn to_u8_array(self) -> [u8; 3] {
        let cr = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let cg = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let cb = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        [cr, cg, cb]
    }
}

/// Samples a normalized temperature `t` in [0.0, 1.0] under WhiteHot palette.
pub fn sample_white_hot(t: f32) -> ColorRgb {
    let val = t.clamp(0.0, 1.0);
    ColorRgb::new(val, val, val)
}

/// Samples a normalized temperature `t` in [0.0, 1.0] under BlackHot palette.
pub fn sample_black_hot(t: f32) -> ColorRgb {
    let val = 1.0 - t.clamp(0.0, 1.0);
    ColorRgb::new(val, val, val)
}

/// Samples a normalized temperature `t` in [0.0, 1.0] under Ironbow palette.
pub fn sample_ironbow(t: f32) -> ColorRgb {
    let x = t.clamp(0.0, 1.0);
    let r = (x * 2.0).clamp(0.0, 1.0);
    let g = ((x - 0.25) * 2.0).clamp(0.0, 1.0);
    let b = ((x - 0.7) * 3.33).max(0.0) + (1.0 - (x * 4.0).min(1.0)) * 0.5;
    ColorRgb::new(r, g, b.clamp(0.0, 1.0))
}

/// Samples a normalized temperature `t` in [0.0, 1.0] under Rainbow palette.
pub fn sample_rainbow(t: f32) -> ColorRgb {
    let x = t.clamp(0.0, 1.0);
    let r = (2.0 * x - 0.5).clamp(0.0, 1.0);
    let g = (1.0 - (2.0 * x - 1.0).abs()).clamp(0.0, 1.0);
    let b = (1.0 - 2.0 * x).clamp(0.0, 1.0);
    ColorRgb::new(r, g, b)
}

/// Samples a normalized temperature `t` in [0.0, 1.0] under GreenHot palette.
pub fn sample_green_hot(t: f32) -> ColorRgb {
    let val = t.clamp(0.0, 1.0);
    ColorRgb::new(val * 0.15, val, val * 0.25)
}

impl ThermalPalette {
    /// Samples RGB color for a given normalized temperature [0.0, 1.0].
    pub fn sample_color(self, t: f32) -> ColorRgb {
        match self {
            Self::WhiteHot => sample_white_hot(t),
            Self::BlackHot => sample_black_hot(t),
            Self::Ironbow => sample_ironbow(t),
            Self::Rainbow => sample_rainbow(t),
            Self::GreenHot => sample_green_hot(t),
        }
    }

    /// Generates a discrete 256-color lookup table (LUT) as RGB triplets.
    pub fn generate_lut_256(self) -> [[u8; 3]; 256] {
        let mut lut = [[0u8; 3]; 256];
        let mut i = 0;
        while i < 256 {
            let t = (i as f32) / 255.0;
            lut[i] = self.sample_color(t).to_u8_array();
            i += 1;
        }
        lut
    }
}

impl ThermalShaderConfig {
    /// Generates GLSL fragment shader source implementing this thermal palette configuration.
    pub fn generate_glsl_fragment_shader(&self) -> String {
        let palette_branch = match self.palette {
            ThermalPalette::WhiteHot => "    vec3 col = vec3(t);\n",
            ThermalPalette::BlackHot => "    vec3 col = vec3(1.0 - t);\n",
            ThermalPalette::Ironbow => {
                "    float r = clamp(t * 2.0, 0.0, 1.0);\n    float g = clamp((t - 0.25) * 2.0, 0.0, 1.0);\n    float b = clamp(max((t - 0.7) * 3.33, 0.0) + (1.0 - min(t * 4.0, 1.0)) * 0.5, 0.0, 1.0);\n    vec3 col = vec3(r, g, b);\n"
            }
            ThermalPalette::Rainbow => {
                "    float r = clamp(2.0 * t - 0.5, 0.0, 1.0);\n    float g = clamp(1.0 - abs(2.0 * t - 1.0), 0.0, 1.0);\n    float b = clamp(1.0 - 2.0 * t, 0.0, 1.0);\n    vec3 col = vec3(r, g, b);\n"
