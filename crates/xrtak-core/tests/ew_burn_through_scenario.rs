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
