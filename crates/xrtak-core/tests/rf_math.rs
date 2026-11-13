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
