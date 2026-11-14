use crate::domain::Affiliation;
use super::affiliation_color::color_for_affiliation;
use super::affiliation_shape::shape_for_affiliation;
use super::billboard_mesh::generate_billboard_quad_mesh;
use super::circle_mesh::generate_circle_marker_mesh;
use super::diamond_mesh::generate_diamond_marker_mesh;
use super::geometry::MarkerGeometry;
use super::shape::MarkerShape;

pub fn generate_marker_geometry(affiliation: Affiliation, size: f32) -> MarkerGeometry {
    let shape = shape_for_affiliation(affiliation);
    let color = color_for_affiliation(affiliation);
    let mesh = match shape {
        MarkerShape::Diamond => generate_diamond_marker_mesh(size),
        MarkerShape::Rectangle | MarkerShape::Square => generate_billboard_quad_mesh(size, size),
        MarkerShape::Circle => generate_circle_marker_mesh(size * 0.5, 16),
    };
    MarkerGeometry {
        shape,
        color,
        mesh,
        scale: [1.0, 1.0, 1.0],
    }
}
