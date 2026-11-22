use xrtak_core::domain::Affiliation;
use xrtak_core::marker::generate_marker_geometry;

// Every marker mesh declares the vertex normal (0, 0, 1), so the first triangle
// must be counter-clockwise when viewed from +Z (positive signed area).
#[test]
fn marker_triangles_face_their_declared_normal() {
    for affil in [
        Affiliation::Hostile,
        Affiliation::Friendly,
        Affiliation::Neutral,
        Affiliation::Unknown,
    ] {
        let mesh = generate_marker_geometry(affil, 2.0).mesh;
        let p: Vec<[f32; 3]> = mesh.indices[..3]
            .iter()
            .map(|&i| mesh.vertices[i as usize].position)
            .collect();
        let z =
            (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[1][1] - p[0][1]) * (p[2][0] - p[0][0]);
        assert!(z > 0.0, "{affil:?}: face normal z = {z}, declared +1");
    }
}
