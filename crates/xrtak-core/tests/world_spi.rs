use xrtak_core::{GeoOrigin, TacticalWorld};

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
