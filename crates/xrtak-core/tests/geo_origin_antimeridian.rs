use xrtak_core::GeoOrigin;

#[test]
fn lla_to_enu_is_continuous_across_the_antimeridian() {
    let origin = GeoOrigin {
        lat: 0.0,
        lon: 179.99,
        alt_m: 0.0,
        extent_m: 1500.0,
    };
    let (east_m, _up, north_m) = origin.lla_to_enu(0.0, -179.99, 0.0);
    assert!(north_m.abs() < 1e-6);
    assert!((east_m - 2226.39).abs() < 5.0, "east_m = {east_m}");
}
