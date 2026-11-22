use xrtak_core::ew::{evaluate_link_jamming, EwTargetBand, JammerProfile, TargetLinkProfile};

#[test]
fn fspl_and_diffraction_canonical_agreement() {
    let d = 1000.0;
    let (f_hz, f_mhz) = (2.4e9, 2400.0);
    let (rf, fz, ew) = (
        xrtak_core::rf::compute_fspl_db(d, f_hz),
        xrtak_core::fresnel_zone::compute_fspl(d, f_hz),
        xrtak_core::ew::compute_fspl_db(d, f_mhz),
    );
    assert_eq!(rf, fz);
    assert!((rf - ew).abs() < 1e-4, "rf vs ew: {rf} vs {ew}");

    let (rf0, fz0, ew0) = (
        xrtak_core::rf::compute_fspl_db(d, 0.0),
        xrtak_core::fresnel_zone::compute_fspl(d, 0.0),
        xrtak_core::ew::compute_fspl_db(d, 0.0),
    );
    assert!(!rf0.is_infinite() && rf0 > 0.0 && rf0 == fz0 && rf0 == ew0);

    let jammer = JammerProfile {
        callsign: "JAM".into(),
        transmitter_power_w: 10.0,
        antenna_gain_dbi: 5.0,
        bandwidth_mhz: 20.0,
        east_m: 0.0,
        north_m: 1000.0,
        up_m: 100.0,
    };
    let link = TargetLinkProfile {
        callsign: "LNK".into(),
        transmitter_power_w: 1.0,
        tx_antenna_gain_dbi: 2.0,
        rx_antenna_gain_dbi: 2.0,
        signal_bandwidth_mhz: 0.0,
        receiver_sensitivity_dbm: -95.0,
        tx_east_m: 0.0,
        tx_north_m: 5000.0,
        tx_up_m: 100.0,
        rx_east_m: 0.0,
        rx_north_m: 0.0,
        rx_up_m: 10.0,
    };
    let res = evaluate_link_jamming(&jammer, &link, EwTargetBand::VideoDownlink, 0.0);
    assert!(res.jammer_to_signal_ratio_db.is_finite());
}
