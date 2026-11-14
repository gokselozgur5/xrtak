use xrtak_core::domain::Affiliation;
use xrtak_core::marker::{
    color_for_affiliation, generate_marker_geometry, generate_spi_pin_marker_geometry,
    shape_for_affiliation, MarkerColor, MarkerShape,
};

#[test]
fn test_marker_color_and_shape_mapping() {
    assert_eq!(color_for_affiliation(Affiliation::Hostile), MarkerColor::RED);
    assert_eq!(color_for_affiliation(Affiliation::Friendly), MarkerColor::BLUE);
    assert_eq!(color_for_affiliation(Affiliation::Neutral), MarkerColor::GREEN);
    assert_eq!(color_for_affiliation(Affiliation::Unknown), MarkerColor::YELLOW);

    assert_eq!(shape_for_affiliation(Affiliation::Hostile), MarkerShape::Diamond);
    assert_eq!(shape_for_affiliation(Affiliation::Friendly), MarkerShape::Rectangle);
    assert_eq!(shape_for_affiliation(Affiliation::Neutral), MarkerShape::Circle);
    assert_eq!(shape_for_affiliation(Affiliation::Unknown), MarkerShape::Square);
}

#[test]
fn test_generate_marker_geometry_affiliations() {
    let hostile = generate_marker_geometry(Affiliation::Hostile, 2.0);
    assert_eq!(hostile.shape, MarkerShape::Diamond);
    assert_eq!(hostile.color, MarkerColor::RED);
    assert_eq!(hostile.mesh.vertices.len(), 4);
    assert_eq!(hostile.mesh.indices.len(), 6);

    let friendly = generate_marker_geometry(Affiliation::Friendly, 2.0);
    assert_eq!(friendly.shape, MarkerShape::Rectangle);
    assert_eq!(friendly.color, MarkerColor::BLUE);
    assert_eq!(friendly.mesh.vertices.len(), 4);
    assert_eq!(friendly.mesh.indices.len(), 6);

    let neutral = generate_marker_geometry(Affiliation::Neutral, 2.0);
    assert_eq!(neutral.shape, MarkerShape::Circle);
    assert_eq!(neutral.color, MarkerColor::GREEN);
    assert!(neutral.mesh.vertices.len() >= 4);
}

#[test]
fn test_generate_spi_pin_marker_geometry() {
    let spi = generate_spi_pin_marker_geometry(1.0, 2.5);
    assert_eq!(spi.color, MarkerColor::CYAN);
    assert_eq!(spi.mesh.vertices.len(), 5);
    assert_eq!(spi.mesh.indices.len(), 18);
    assert_eq!(spi.mesh.vertices[0].position, [0.0, 0.0, 0.0]);
}
