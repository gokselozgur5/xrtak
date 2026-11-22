use std::ptr;
use xrtak_core::ffi::{
    xrtak_lla_to_enu, xrtak_sample_relief_m, XRTAK_ERR_INVALID, XRTAK_ERR_NULL, XRTAK_OK,
};

#[test]
fn lla_to_enu_projects_against_caller_origin() {
    let mut enu = [99.0_f64; 3];
    let (lat, lon, alt) = (-33.8688, 151.2093, 42.0);
    let rc = unsafe { xrtak_lla_to_enu(lat, lon, alt, lat, lon, alt, enu.as_mut_ptr()) };
    assert_eq!(rc, XRTAK_OK);
    assert!(enu[0].abs() < 1e-9 && enu[1].abs() < 1e-9 && enu[2].abs() < 1e-9);

    let rc = unsafe { xrtak_lla_to_enu(0.0, 0.0, 100.0, 0.01, 0.02, 150.0, enu.as_mut_ptr()) };
    assert_eq!(rc, XRTAK_OK);
    assert!((enu[0] - 2228.2568).abs() < 1e-6);
    assert!((enu[1] - 50.0).abs() < 1e-9);
    assert!((enu[2] - 1105.731).abs() < 1e-6);
}

#[test]
fn ffi_rejects_null_and_invalid_inputs() {
    let rc_null = unsafe { xrtak_lla_to_enu(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, ptr::null_mut()) };
    assert_eq!(rc_null, XRTAK_ERR_NULL);
    let mut enu = [-7.0_f64; 3];
    let rc_nan = unsafe { xrtak_lla_to_enu(f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0, enu.as_mut_ptr()) };
    let rc_lat = unsafe { xrtak_lla_to_enu(0.0, 0.0, 0.0, 91.0, 0.0, 0.0, enu.as_mut_ptr()) };
    let rc_lon = unsafe { xrtak_lla_to_enu(0.0, -181.0, 0.0, 0.0, 0.0, 0.0, enu.as_mut_ptr()) };
    assert_eq!(
        (rc_nan, rc_lat, rc_lon),
        (XRTAK_ERR_INVALID, XRTAK_ERR_INVALID, XRTAK_ERR_INVALID)
    );
    assert_eq!(enu, [-7.0, -7.0, -7.0]);
    assert!(xrtak_sample_relief_m(f64::NAN, 0.0).is_nan());
}

#[test]
fn crate_ships_c_library_types_and_header() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let manifest = std::fs::read_to_string(format!("{dir}/Cargo.toml")).unwrap();
    for kind in ["\"rlib\"", "\"cdylib\"", "\"staticlib\""] {
        assert!(manifest.contains(kind), "missing crate-type {kind}");
    }
    let header = std::fs::read_to_string(format!("{dir}/include/xrtak.h")).unwrap();
    assert!(header.contains("double xrtak_sample_relief_m(double east_m, double north_m);"));
    assert!(header.contains("int32_t xrtak_lla_to_enu("));
}
