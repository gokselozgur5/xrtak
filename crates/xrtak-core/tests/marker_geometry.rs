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
