use xrtak_core::{GeoOrigin, TacticalWorld};

#[test]
fn test_sim_disabled_removes_demo_tactical_zones() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    assert_eq!(world.tactical_zones.len(), 2);
    world.set_sim_enabled(false);
    assert!(world.tactical_zones.is_empty());
    world.set_sim_enabled(true);
    assert_eq!(world.tactical_zones.len(), 2);
}
