use xrtak_core::rf::*;

#[test]
fn fspl_at_reference_frequency_and_distance() {
    let fspl = compute_fspl_db(1000.0, 2_400_000_000.0);
    let expected = 100.05;
    assert!(
        (fspl - expected).abs() < 0.05,
        "FSPL {fspl} not within 0.05 dB of {expected}"
    );

    let fspl_short = compute_fspl_db(0.5, 2_400_000_000.0);
    let fspl_one_m = compute_fspl_db(1.0, 2_400_000_000.0);
    assert_eq!(fspl_short, fspl_one_m);
}

#[test]
fn fresnel_radius_matches_closed_form() {
    let freq = 2_400_000_000.0;
    let lambda = 299_792_458.0 / freq;
    let d1: f64 = 500.0;
    let d2: f64 = 500.0;
    let analytic: f64 = ((lambda * d1 * d2) / (d1 + d2)).sqrt();

    let computed = compute_fresnel_radius_m(d1, d2, freq);
    assert!(
        (computed - analytic).abs() < 1e-9,
        "Fresnel radius {computed} does not match analytic {analytic}"
    );
    assert_eq!(compute_fresnel_radius_m(0.0, 0.0, freq), 0.0);
}

#[test]
fn knife_edge_diffraction_reference_values() {
    assert_eq!(compute_knife_edge_diffraction_db(-0.78), 0.0);
    assert_eq!(compute_knife_edge_diffraction_db(-1.5), 0.0);

    let loss_v0 = compute_knife_edge_diffraction_db(0.0);
    assert!(
        (loss_v0 - 6.03).abs() < 0.1,
        "Diffraction at v=0 was {loss_v0}, expected ~6.03 dB"
    );

    let loss_v1 = compute_knife_edge_diffraction_db(1.0);
    assert!(loss_v1 > loss_v0);
}

#[test]
fn diffraction_v_and_band_parameters() {
    let v_flat = compute_diffraction_v(0.0, 500.0, 500.0, 2.4e9);
    assert_eq!(v_flat, 0.0);
    assert_eq!(compute_diffraction_v(10.0, 0.0, 500.0, 2.4e9), -1.0);

    assert_eq!(RfBand::UHF900.frequency_hz(), 900_000_000.0);
    assert_eq!(RfBand::ISM2400.frequency_hz(), 2_400_000_000.0);
    assert_eq!(RfBand::SHF5800.frequency_hz(), 5_800_000_000.0);
    assert_eq!(RfBand::UHF900.default_tx_power_dbm(), 30.0);
    assert_eq!(RfBand::UHF900.default_rx_sensitivity_dbm(), -110.0);
}
