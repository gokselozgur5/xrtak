use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarkerColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl MarkerColor {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl MarkerColor {
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.5, 1.0);
    pub const GREEN: Self = Self::rgb(0.0, 0.9, 0.1);
    pub const YELLOW: Self = Self::rgb(1.0, 0.85, 0.0);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
}
