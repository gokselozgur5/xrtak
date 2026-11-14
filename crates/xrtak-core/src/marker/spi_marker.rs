use super::color::MarkerColor;
use super::geometry::MarkerGeometry;
use super::pyramid_mesh::generate_pyramid_marker_mesh;
use super::shape::MarkerShape;

pub fn generate_spi_pin_marker_geometry(base_size: f32, height: f32) -> MarkerGeometry {
    let mesh = generate_pyramid_marker_mesh(base_size, height);
    MarkerGeometry {
        shape: MarkerShape::Square,
        color: MarkerColor::CYAN,
        mesh,
        scale: [1.0, 1.0, 1.0],
    }
}
