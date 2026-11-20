use xrtak_core::{GeoOrigin, SapientDetectionReport, TacticalWorld};

#[test]
fn sapient_detection_does_not_slew_absent_demo_uav() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    world.set_sim_enabled(false);
    let report = SapientDetectionReport {
        node_id: "N-1".into(),
        object_id: "O-1".into(),
        classification: "TANK".into(),
        confidence: 0.9,
        east_m: 100.0,
        north_m: 200.0,
        alt_offset_m: None,
        speed_mps: None,
        course_deg: None,
        auto_slew_sensor_uid: None,
    };
    world.ingest_sapient_detection(&report, 1_000).expect("ingest");
    let uids: Vec<&String> = world.entities.keys().collect();
    assert!(!world.entities.contains_key("UAV-FALCON-01.SPI1"), "entities: {uids:?}");
    assert!(world.spi_locks.is_empty(), "locks: {:?}", world.spi_locks);
}
