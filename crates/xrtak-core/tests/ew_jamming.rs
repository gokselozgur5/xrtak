use xrtak_core::ew::watts_to_dbm;

#[test]
fn test_watts_to_dbm_conversions() {
    assert_eq!(watts_to_dbm(1.0), 30.0);
    assert_eq!(watts_to_dbm(0.001), 0.0);
    assert_eq!(watts_to_dbm(0.0), -150.0);
    assert_eq!(watts_to_dbm(10.0), 40.0);
}
