use super::mesh::{MarkerMesh, Vertex3D};

pub fn generate_circle_marker_mesh(radius: f32, segments: usize) -> MarkerMesh {
    let segs = if segments < 3 { 3 } else { segments };
    let mut vertices = Vec::with_capacity(segs + 1);
    let mut indices = Vec::with_capacity(segs * 3);
    let n = [0.0, 0.0, 1.0];
    vertices.push(Vertex3D::new([0.0, 0.0, 0.0], n, [0.5, 0.5]));
    let step = std::f32::consts::TAU / (segs as f32);
    for i in 0..segs {
        let angle = (i as f32) * step;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let u = 0.5 + 0.5 * angle.cos();
        let v = 0.5 + 0.5 * angle.sin();
        vertices.push(Vertex3D::new([x, y, 0.0], n, [u, v]));
        let curr = (i + 1) as u32;
        let next = if i + 1 == segs { 1 } else { (i + 2) as u32 };
        indices.extend_from_slice(&[0, curr, next]);
    }
    MarkerMesh { vertices, indices }
}
