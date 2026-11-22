use xrtak_core::opord::build_opord_bundle;

#[test]
fn opord_bundle_timestamp_is_not_a_fixed_literal() {
    let bundle = build_opord_bundle("ANVIL", (39.0, 32.0, 900.0), &[], 0, 0);
    assert_ne!(bundle.generated_utc, "2026-11-07T11:00:00Z");
}

#[test]
fn opord_origin_formats_hemispheres_from_coordinate_signs() {
    let bundle = build_opord_bundle("SOUTHERN_CROSS", (-33.9, -70.6, 500.0), &[], 0, 0);
    assert!(
        bundle.opord_markdown.contains("33.90000S, 70.60000W"),
        "expected formatted hemispheres '33.90000S, 70.60000W', got:\n{}",
        bundle.opord_markdown
    );
    assert!(
        bundle.kml_scene.contains("33.90000S, 70.60000W"),
        "expected kml description to format hemispheres, got:\n{}",
        bundle.kml_scene
    );
}
