use xrtak_core::{GeoOrigin, SapientDetectionReport, TacticalWorld};

#[test]
fn slew_sensor_creates_linked_spi_and_lock() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    world.step_simulation(0.1, 1000);
    assert!(world.entities.contains_key("UAV-FALCON-01"));

    let spi_res = world.slew_sensor_to_spi("UAV-FALCON-01", 120.0, 340.0, 2000);
    assert!(spi_res.is_ok());

    let spi_uid = "UAV-FALCON-01.SPI1";
    let spi_ent = world.entities.get(spi_uid).expect("SPI entity exists");
    assert_eq!(spi_ent.cot_type, "b-m-p-s-p-i");
    assert_eq!(spi_ent.parent_uid.as_deref(), Some("UAV-FALCON-01"));
    assert!((spi_ent.east_m - 120.0).abs() < 0.1);
    assert!((spi_ent.north_m - 340.0).abs() < 0.1);

    let lock = world.spi_locks.get("UAV-FALCON-01").copied().unwrap();
    assert_eq!(lock, (spi_ent.east_m, spi_ent.up_m, spi_ent.north_m));
}

#[test]
fn clear_spi_lock_removes_entity_and_lock() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    world.step_simulation(0.1, 1000);
    let res = world.slew_sensor_to_spi("UAV-FALCON-01", 100.0, 200.0, 2000);
    assert!(res.is_ok());
    assert!(world.entities.contains_key("UAV-FALCON-01.SPI1"));
    assert!(world.spi_locks.contains_key("UAV-FALCON-01"));

    world.clear_spi_lock("UAV-FALCON-01");
    assert!(!world.entities.contains_key("UAV-FALCON-01.SPI1"));
    assert!(!world.spi_locks.contains_key("UAV-FALCON-01"));
}

#[test]
fn sapient_high_confidence_auto_slews_falcon() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    world.step_simulation(0.1, 1000);
    let report = SapientDetectionReport {
        node_id: "RADAR-1".to_string(),
        object_id: "THREAT-1".to_string(),
        classification: "HOSTILE_ARMOR".to_string(),
        confidence: 0.95,
        east_m: -150.0,
        north_m: 220.0,
        alt_offset_m: Some(0.0),
        speed_mps: Some(8.0),
        course_deg: Some(45.0),
        auto_slew_sensor_uid: None,
    };
    let (_, ent) = world.ingest_sapient_detection(&report, 2000).unwrap();
    assert_eq!(ent.uid, "SAPIENT-THREAT-1");
    assert!(world.entities.contains_key("UAV-FALCON-01.SPI1"));
    assert!(world.spi_locks.contains_key("UAV-FALCON-01"));
}

#[test]
fn sapient_auto_slew_explicit_sensor_and_low_confidence() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    let mut report = SapientDetectionReport {
        node_id: "RADAR-1".to_string(),
        object_id: "THREAT-2".to_string(),
        classification: "HOSTILE_ARMOR".to_string(),
        confidence: 0.50,
        east_m: -100.0,
        north_m: 100.0,
        alt_offset_m: None,
        speed_mps: None,
        course_deg: None,
        auto_slew_sensor_uid: None,
    };
    let _ = world.ingest_sapient_detection(&report, 3000).unwrap();
    assert!(!world.spi_locks.contains_key("UAV-FALCON-01"));

    report.object_id = "THREAT-3".to_string();
    report.auto_slew_sensor_uid = Some("UAV-VALKYRIE-02".to_string());
    let _ = world.ingest_sapient_detection(&report, 3100).unwrap();
    assert!(world.spi_locks.contains_key("UAV-VALKYRIE-02"));
}
