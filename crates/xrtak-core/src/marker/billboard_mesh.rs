use super::mesh::{MarkerMesh, Vertex3D};

pub fn generate_billboard_quad_mesh(width: f32, height: f32) -> MarkerMesh {
    let hw = width * 0.5;
    let hh = height * 0.5;
    let n = [0.0, 0.0, 1.0];
    let vertices = vec![
        Vertex3D::new([-hw, hh, 0.0], n, [0.0, 1.0]),
        Vertex3D::new([hw, hh, 0.0], n, [1.0, 1.0]),
        Vertex3D::new([hw, -hh, 0.0], n, [1.0, 0.0]),
        Vertex3D::new([-hw, -hh, 0.0], n, [0.0, 0.0]),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    MarkerMesh { vertices, indices }
}
