use super::mesh::{MarkerMesh, Vertex3D};

pub fn generate_pyramid_marker_mesh(base_size: f32, height: f32) -> MarkerMesh {
    let half = base_size * 0.5;
    let apex = [0.0, 0.0, 0.0];
    let top = height;
    let vertices = vec![
        Vertex3D::new(apex, [0.0, 0.0, -1.0], [0.5, 0.5]),
        Vertex3D::new([-half, half, top], [-0.5, 0.5, 0.5], [0.0, 1.0]),
        Vertex3D::new([half, half, top], [0.5, 0.5, 0.5], [1.0, 1.0]),
        Vertex3D::new([half, -half, top], [0.5, -0.5, 0.5], [1.0, 0.0]),
        Vertex3D::new([-half, -half, top], [-0.5, -0.5, 0.5], [0.0, 0.0]),
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1,
        1, 3, 2, 1, 4, 3,
    ];
    MarkerMesh { vertices, indices }
}
