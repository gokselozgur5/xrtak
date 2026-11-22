use xrtak_core::geo::ProceduralReliefModel;
use xrtak_core::mission::{
    evaluate_mission_corridor, export_atak_route_cot_xml, RawMissionWaypoint,
};
use xrtak_core::GeoOrigin;

#[test]
fn route_event_carries_required_time_attributes() {
    let xml = export_atak_route_cot_xml(
        "RTE-1",
        "ALPHA",
        &GeoOrigin::default(),
        &ProceduralReliefModel,
        &[(0.0, 0.0, 100.0), (200.0, 100.0, 100.0)],
    );
    let event_tag = &xml[..xml.find('>').expect("event start tag")];
    for attr in [" time=\"", " start=\"", " stale=\""] {
        assert!(event_tag.contains(attr), "missing{attr} in {event_tag}");
    }
}

#[test]
fn evaluate_mission_corridor_preserves_negative_flight_up_below_origin() {
    let origin = GeoOrigin {
        lat: 37.7749,
        lon: -122.4194,
        alt_m: 850.0,
        extent_m: 1500.0,
    };
    let waypoints = vec![RawMissionWaypoint {
        name: "WP-LOW".to_string(),
        lat: 37.7749,
        lon: -122.4194,
        alt_m: 400.0,
        alt_is_relative: false,
    }];
    let result = evaluate_mission_corridor(
        "TEST-BELOW-ORIGIN",
        &waypoints,
        &origin,
        &ProceduralReliefModel,
        100.0,
        50.0,
    );
    assert_eq!(result.waypoints.len(), 1);
    assert_eq!(result.waypoints[0].flight_up_m, -450.0);
}
