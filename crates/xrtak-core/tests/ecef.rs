use xrtak_core::geodesy::ecef::WGS84_B;
use xrtak_core::geodesy::{ecef_to_geodetic, EcefCoord};

#[test]
fn test_ecef_to_geodetic_near_polar_axis_precision() {
    let ecef = EcefCoord::new(0.01, 0.0, WGS84_B);
    let geo = ecef_to_geodetic(&ecef);
    assert!(
        geo.alt_m.abs() < 1e-4,
        "altitude near polar axis should be near 0 m, got {}",
        geo.alt_m
    );
}
