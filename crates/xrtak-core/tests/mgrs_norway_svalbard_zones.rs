use xrtak_core::mgrs::{GeodeticCoord, MgrsPrecision};

fn mgrs_1m(lat: f64, lon: f64) -> String {
    GeodeticCoord::new(lat, lon)
        .and_then(|g| g.to_mgrs(MgrsPrecision::OneMeter))
        .map(|m| m.to_compact_string())
        .unwrap_or_else(|e| format!("error: {e}"))
}

#[test]
fn bergen_is_in_grid_zone_32v() {
    // 60.3913 N, 5.3221 E lies in the widened zone 32V (3 E to 12 E between 56 N and 64 N).
    assert_eq!(mgrs_1m(60.3913, 5.3221), "32VKN9735300648");
}

#[test]
fn ny_alesund_is_in_grid_zone_33x() {
    // 78.9235 N, 11.9100 E lies in zone 33X (9 E to 21 E between 72 N and 84 N).
    assert_eq!(mgrs_1m(78.9235, 11.9100), "33XVH3375763189");
}
