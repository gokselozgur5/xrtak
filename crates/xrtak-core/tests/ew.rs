use xrtak_core::ew::compute_burn_through_range;

#[test]
fn burn_through_range_matches_standoff_jamming_equation() {
    let bt = compute_burn_through_range(
        "RADAR", 50_000.0, 30.0, 2.0, "JAMMER", 500.0, 10.0, 50_000.0, 12_000.0,
    );
    let x = 50_000.0 * 1000.0 * 2.0 * 50_000.0_f64.powi(2)
        / (4.0 * std::f64::consts::PI * 500.0 * 10.0);
    let expected = x.powf(0.25);
    assert!(
        (bt.burn_through_range_m - expected).abs() < 1.0,
        "got {} m, expected {:.1} m",
        bt.burn_through_range_m,
        expected
    );
    assert!(!bt.is_burned_through);
}
