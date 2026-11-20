use xrtak_core::route_sequencer::{
    generate_cot_route_xml, sequence_route_plan, RoutePlan, RouteWaypoint,
};

#[test]
fn route_event_carries_the_mandatory_cot_time_attributes() {
    let plan = RoutePlan {
        route_uid: "RT-1".into(),
        callsign: "EAGLE-1".into(),
        default_speed_mps: 20.0,
        start_time_epoch_sec: 1_710_000_000.0,
        waypoints: vec![
            RouteWaypoint::new("START", 35.0, -115.0, 500.0),
            RouteWaypoint::new("LZ", 35.05, -115.05, 520.0),
        ],
    };
    let xml = generate_cot_route_xml(&plan, &sequence_route_plan(&plan));
    let event_tag = &xml[..xml.find('>').unwrap()];

    for attr in [" time=\"", " start=\"", " stale=\""] {
        assert!(event_tag.contains(attr), "missing {attr} in {event_tag}");
    }
}
