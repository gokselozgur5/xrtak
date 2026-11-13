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

use xrtak_core::ew::JammerProfile;

#[allow(dead_code)]
fn make_jammer(p_w: f64) -> JammerProfile {
    JammerProfile {
        callsign: "JAM".into(),
        transmitter_power_w: p_w,
        antenna_gain_dbi: 10.0,
        bandwidth_mhz: 20.0,
        east_m: 0.0,
        north_m: 100.0,
        up_m: 50.0,
    }
}

use xrtak_core::ew::TargetLinkProfile;

#[allow(dead_code)]
fn make_link() -> TargetLinkProfile {
    TargetLinkProfile {
        callsign: "LINK".into(),
        transmitter_power_w: 1.0,
        tx_antenna_gain_dbi: 2.0,
        rx_antenna_gain_dbi: 2.0,
        signal_bandwidth_mhz: 10.0,
        receiver_sensitivity_dbm: -95.0,
        tx_east_m: 0.0,
        tx_north_m: 5000.0,
        tx_up_m: 100.0,
        rx_east_m: 0.0,
        rx_north_m: 0.0,
        rx_up_m: 10.0,
    }
}

use xrtak_core::ew::{evaluate_link_jamming, EwTargetBand};

#[test]
fn test_high_js_marks_link_denied() {
    let jammer = make_jammer(100.0);
    let link = make_link();
    let res = evaluate_link_jamming(&jammer, &link, EwTargetBand::VideoDownlink, 0.0);
    assert!(res.is_denied);
    assert!(res.jammer_to_signal_ratio_db > 0.0);

    let weak_jammer = make_jammer(0.0001);
    let clear_res = evaluate_link_jamming(&weak_jammer, &link, EwTargetBand::VideoDownlink, 0.0);
    assert!(!clear_res.is_denied);
}
