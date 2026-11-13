use xrtak_core::ew::watts_to_dbm;

#[test]
fn test_watts_to_dbm_conversions() {
    assert_eq!(watts_to_dbm(1.0), 30.0);
    assert_eq!(watts_to_dbm(0.001), 0.0);
    assert_eq!(watts_to_dbm(0.0), -150.0);
    assert_eq!(watts_to_dbm(10.0), 40.0);
}

#[test]
fn test_ew_fspl_mhz_equals_rf_fspl_hz() {
    let ew_loss = xrtak_core::ew::compute_fspl_db(1000.0, 2400.0);
    let rf_loss = xrtak_core::rf::compute_fspl_db(1000.0, 2_400_000_000.0);
    assert!((ew_loss - rf_loss).abs() < 0.05);
}
