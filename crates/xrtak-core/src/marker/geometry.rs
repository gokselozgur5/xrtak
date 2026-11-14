use serde::{Deserialize, Serialize};
use super::color::MarkerColor;
use super::mesh::MarkerMesh;
use super::shape::MarkerShape;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerGeometry {
    pub shape: MarkerShape,
    pub color: MarkerColor,
    pub mesh: MarkerMesh,
    pub scale: [f32; 3],
}
