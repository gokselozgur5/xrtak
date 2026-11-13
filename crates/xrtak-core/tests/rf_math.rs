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
