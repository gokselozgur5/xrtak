use xrtak_core::{GeoOrigin, TacticalWorld};

fn event(time: &str, lat: &str) -> String {
    format!(
        r#"<event version="2.0" uid="U-1" type="a-f-G" time="{time}" start="{time}" stale="2099-01-01T00:00:00Z" how="m-g"><point lat="{lat}" lon="32.85" hae="900"/></event>"#
    )
}

#[test]
fn older_event_does_not_overwrite_newer_position() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    world.set_sim_enabled(false);
    world
        .ingest_cot_xml(&event("2026-11-18T10:00:10Z", "39.9300"), 1_000)
        .expect("newer");
    let _ = world.ingest_cot_xml(&event("2026-11-18T10:00:00Z", "39.9100"), 1_050);
    let lat = world.entities["U-1"].lat;
    assert!((lat - 39.93).abs() < 1e-9, "position rolled back to lat={lat}");
}
