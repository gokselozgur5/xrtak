use xrtak_core::ew::compute_burn_through_range;

#[test]
fn burn_through_shrinks_as_jammer_power_rises() {
    let r_low_jam = compute_burn_through_range(
        "RADAR-1", 10_000.0, 30.0, 2.0, "JAMMER-1", 100.0, 10.0, 50_000.0, 10_000.0,
    );
    let r_med_jam = compute_burn_through_range(
        "RADAR-1", 10_000.0, 30.0, 2.0, "JAMMER-1", 500.0, 10.0, 50_000.0, 10_000.0,
    );
    let r_high_jam = compute_burn_through_range(
        "RADAR-1", 10_000.0, 30.0, 2.0, "JAMMER-1", 2_000.0, 10.0, 50_000.0, 10_000.0,
    );

    assert!(r_low_jam.burn_through_range_m > r_med_jam.burn_through_range_m);
    assert!(r_med_jam.burn_through_range_m > r_high_jam.burn_through_range_m);
}

#[test]
fn burn_through_grows_as_radar_power_rises() {
    let r_low_pwr = compute_burn_through_range(
        "RADAR-1", 1_000.0, 30.0, 2.0, "JAMMER-1", 500.0, 10.0, 50_000.0, 10_000.0,
    );
    let r_med_pwr = compute_burn_through_range(
        "RADAR-1", 10_000.0, 30.0, 2.0, "JAMMER-1", 500.0, 10.0, 50_000.0, 10_000.0,
    );
    let r_high_pwr = compute_burn_through_range(
        "RADAR-1", 50_000.0, 30.0, 2.0, "JAMMER-1", 500.0, 10.0, 50_000.0, 10_000.0,
    );

    assert!(r_low_pwr.burn_through_range_m < r_med_pwr.burn_through_range_m);
    assert!(r_med_pwr.burn_through_range_m < r_high_pwr.burn_through_range_m);
}
