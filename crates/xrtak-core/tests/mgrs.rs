use xrtak_core::mgrs::{GeodeticCoord, MgrsPrecision};

#[test]
fn single_digit_grid_zone_is_zero_padded_in_compact_and_spaced_output() {
    let coord = GeodeticCoord::new(45.0, -153.0).expect("valid coordinate in zone 5");
    let mgrs = coord
        .to_mgrs(MgrsPrecision::OneMeter)
        .expect("MGRS converted");
    assert_eq!(mgrs.gzd.zone, 5);
    assert!(mgrs.to_compact_string().starts_with("05"));
    assert!(mgrs.to_spaced_string().starts_with("05"));
}
