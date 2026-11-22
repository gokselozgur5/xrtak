use super::mesh::{MarkerMesh, Vertex3D};

pub fn generate_diamond_marker_mesh(size: f32) -> MarkerMesh {
    let half = size * 0.5;
    let n = [0.0, 0.0, 1.0];
    let vertices = vec![
        Vertex3D::new([0.0, half, 0.0], n, [0.5, 1.0]),
        Vertex3D::new([half, 0.0, 0.0], n, [1.0, 0.5]),
        Vertex3D::new([0.0, -half, 0.0], n, [0.5, 0.0]),
        Vertex3D::new([-half, 0.0, 0.0], n, [0.0, 0.5]),
    ];
    let indices = vec![0, 2, 1, 0, 3, 2];
    MarkerMesh { vertices, indices }
}
