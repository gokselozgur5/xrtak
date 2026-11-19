use xrtak_core::{GeoOrigin, TacticalWorld};

#[test]
fn stale_event_does_not_stay_in_world() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    world.set_sim_enabled(false);
    let xml = r#"<event version="2.0" uid="GHOST-1" type="a-f-G-U-C" time="2020-01-01T00:00:00Z" start="2020-01-01T00:00:00Z" stale="2020-01-01T00:01:00Z" how="m-g"><point lat="39.92" lon="32.85" hae="900"/></event>"#;
    let now_ms = 1_794_996_000_000; // 2026-11-18T10:00:00Z
    let _ = world.ingest_cot_xml(xml, now_ms);
    world.step_simulation(0.05, now_ms + 3_600_000);
    assert!(
        !world.entities.contains_key("GHOST-1"),
        "event that went stale in 2020 is still tracked in 2026"
    );
}
