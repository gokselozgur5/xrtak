use xrtak_core::tiles::*;

#[test]
fn tile_xy_at_origin_and_zoom_zero() {
    assert_eq!(latlon_to_tile_xy(0.0, 0.0, 0), (0, 0));
    assert_eq!(latlon_to_tile_xy(0.0, 0.0, 1), (1, 1));
}

#[test]
fn tile_xy_clamps_poles_and_antimeridian() {
    assert_eq!(latlon_to_tile_xy(90.0, -180.0, 2), (0, 0));
    assert_eq!(latlon_to_tile_xy(-90.0, 180.0, 2), (3, 3));
}

#[test]
fn quadkey_matches_bing_reference() {
    assert_eq!(tile_xy_to_quadkey(3, 5, 3), "213");
    assert_eq!(tile_xy_to_quadkey(0, 0, 0), "");
}

#[test]
fn parent_tile_shifts_and_ignores_higher_target() {
    assert_eq!(parent_tile_xy(5, 7, 3, 1), (1, 1, 1));
    assert_eq!(parent_tile_xy(5, 7, 3, 5), (5, 7, 3));
}

#[test]
fn bing_subdomain_stays_in_range() {
    assert_eq!(bing_subdomain(3, 5), 0);
    assert_eq!(bing_subdomain(1, 2), 3);
}

#[test]
fn provider_ids_parse_with_default() {
    assert_eq!(
        ImageryProvider::from_id(" BING "),
        ImageryProvider::BingAerial
    );
    assert_eq!(
        ImageryProvider::from_id("S2"),
        ImageryProvider::Sentinel2Cloudless
    );
    assert_eq!(
        ImageryProvider::from_id("topo"),
        ImageryProvider::OpenTopoMap
    );
    assert_eq!(
        ImageryProvider::from_id("nope"),
        ImageryProvider::EsriWorldImagery
    );
}

#[test]
fn tile_urls_order_coordinates_per_provider() {
    let esri = ImageryProvider::EsriWorldImagery.tile_url(1, 2, 3);
    assert!(esri.ends_with("/tile/3/2/1"));
    let topo = ImageryProvider::OpenTopoMap.tile_url(1, 2, 3);
    assert_eq!(topo, "https://tile.opentopomap.org/3/1/2.png");
}

#[test]
fn placeholder_detection_edge_cases() {
    assert!(is_placeholder_rgb_tile(&[]));
    assert!(!is_placeholder_rgb_tile(&[0u8; 48]));
}

#[test]
fn resolve_clamps_zoom_and_caps_fallback() {
    let spec = resolve_tile_with_fallback(ImageryProvider::OpenTopoMap, 10.0, 20.0, 99);
    assert_eq!(spec.zoom, 20);
    assert_eq!(spec.fallback_zoom, 13);
    let low = resolve_tile_with_fallback(ImageryProvider::OpenTopoMap, 10.0, 20.0, 5);
    assert_eq!(low.fallback_zoom, 5);
    assert_eq!(low.primary_url, low.fallback_url);
}
