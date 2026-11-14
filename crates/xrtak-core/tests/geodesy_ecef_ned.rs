use xrtak_core::geodesy::{
    ecef_to_geodetic, ecef_to_ned, geodetic_to_ecef, ned_to_ecef, EcefCoord, GeodeticCoord,
    NedCoord, NedOrigin,
};

#[test]
fn test_geodetic_to_ecef_reference_equator() {
    let geo = GeodeticCoord::new(0.0, 0.0, 0.0);
    let ecef = geodetic_to_ecef(&geo);
    assert!((ecef.x_m - 6_378_137.0).abs() < 1e-4);
    assert!(ecef.y_m.abs() < 1e-4);
    assert!(ecef.z_m.abs() < 1e-4);
}

#[test]
fn test_geodetic_to_ecef_reference_north_pole() {
    let geo = GeodeticCoord::new(90.0, 0.0, 0.0);
    let ecef = geodetic_to_ecef(&geo);
    assert!(ecef.x_m.abs() < 1e-4);
    assert!(ecef.y_m.abs() < 1e-4);
    assert!((ecef.z_m - 6_356_752.3142).abs() < 1e-3);
}

#[test]
fn test_geodetic_to_ecef_roundtrip_ankara() {
    let ankara = GeodeticCoord::new(39.9208, 32.8541, 850.0);
    let ecef = geodetic_to_ecef(&ankara);
    let recovered = ecef_to_geodetic(&ecef);
    assert!((recovered.lat_deg - ankara.lat_deg).abs() < 1e-7);
    assert!((recovered.lon_deg - ankara.lon_deg).abs() < 1e-7);
    assert!((recovered.alt_m - ankara.alt_m).abs() < 1e-4);
}
