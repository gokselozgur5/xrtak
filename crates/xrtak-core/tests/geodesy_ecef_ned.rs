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
    assert!((ecef.z_m - 6_356_752.314_2).abs() < 1e-3);
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

#[test]
fn test_ecef_to_ned_origin_displacement() {
    let origin = NedOrigin::new(39.9208, 32.8541, 850.0);
    let ned_origin = ecef_to_ned(&origin.ecef, &origin);
    assert!(ned_origin.north_m.abs() < 1e-6);
    assert!(ned_origin.east_m.abs() < 1e-6);
    assert!(ned_origin.down_m.abs() < 1e-6);
}

#[test]
fn test_ned_ecef_roundtrip_and_cardinal_offsets() {
    let origin = NedOrigin::new(39.9208, 32.8541, 850.0);
    let original_ned = NedCoord::new(1500.0, -800.0, -250.0);
    let ecef = ned_to_ecef(&original_ned, &origin);
    let recovered_ned = ecef_to_ned(&ecef, &origin);

    assert!((recovered_ned.north_m - original_ned.north_m).abs() < 1e-4);
    assert!((recovered_ned.east_m - original_ned.east_m).abs() < 1e-4);
    assert!((recovered_ned.down_m - original_ned.down_m).abs() < 1e-4);
}

#[test]
fn test_ecef_coord_direct_usage() {
    let p = EcefCoord::new(100.0, 200.0, 300.0);
    assert_eq!(p.x_m, 100.0);
    assert_eq!(p.y_m, 200.0);
    assert_eq!(p.z_m, 300.0);
}
