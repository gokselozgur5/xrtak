use xrtak_core::ew::BurnThroughScenario;

#[test]
fn test_burn_through_scenario_construction() {
    let scenario = BurnThroughScenario::new(
        "AN-MPQ-64",
        10_000.0,
        30.0,
        1.0,
        "KRASUKHA",
        200.0,
        10.0,
        50_000.0,
        15_000.0,
    );
    assert_eq!(scenario.radar_callsign, "AN-MPQ-64");
    assert_eq!(scenario.jammer_callsign, "KRASUKHA");
    assert_eq!(scenario.p_radar_w, 10_000.0);
}

#[test]
fn test_burn_through_scenario_evaluate_method() {
    let scenario = BurnThroughScenario::new(
        "AN-MPQ-64",
        10_000.0,
        30.0,
        1.0,
        "KRASUKHA",
        200.0,
        10.0,
        50_000.0,
        15_000.0,
    );
    let assessment = scenario.evaluate();
    assert_eq!(assessment.radar_callsign, "AN-MPQ-64");
    assert_eq!(assessment.jammer_callsign, "KRASUKHA");
    assert!((assessment.burn_through_range_m - 998.7).abs() < 0.1);
    assert_eq!(assessment.target_rcs_dbsm, 0.0);
    assert!(!assessment.is_burned_through);

    let scenario_inside = BurnThroughScenario::new(
        "AN-MPQ-64",
        10_000.0,
        30.0,
        1.0,
        "KRASUKHA",
        200.0,
        10.0,
        50_000.0,
        500.0,
    );
    let assessment_inside = scenario_inside.evaluate();
    assert!((assessment_inside.burn_through_range_m - 998.7).abs() < 0.1);
    assert!(assessment_inside.is_burned_through);
}
