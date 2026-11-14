use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarkerColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
