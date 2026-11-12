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
