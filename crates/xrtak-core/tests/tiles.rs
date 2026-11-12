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
