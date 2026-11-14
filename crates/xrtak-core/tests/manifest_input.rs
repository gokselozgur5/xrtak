use xrtak_core::manifest::ManifestInput;
use xrtak_core::tiles::ImageryProvider;

#[test]
fn test_manifest_input_struct() {
    let input = ManifestInput {
        name: "ANKARA",
        provider: ImageryProvider::BingAerial,
        south_deg: 39.9,
        west_deg: 32.8,
        north_deg: 40.0,
        east_deg: 32.9,
        min_zoom: 10,
        max_zoom: 14,
    };
    let m = input.build();
    assert_eq!(m.name, "ANKARA");
    assert_eq!(m.min_zoom, 10);
    assert_eq!(m.max_zoom, 14);
}
