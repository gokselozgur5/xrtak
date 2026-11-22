use xrtak_core::route_sequencer::{
    evaluate_route_progress, generate_cot_route_xml, sequence_route_plan, RoutePlan, RouteWaypoint,
};

fn sample_plan(n: usize) -> RoutePlan {
    RoutePlan {
        route_uid: "R".into(),
        callsign: "C".into(),
        default_speed_mps: 10.0,
        start_time_epoch_sec: 0.0,
        waypoints: (0..n)
            .map(|i| RouteWaypoint::new(format!("W{i}"), 0.0, i as f64 * 0.01, 0.0))
            .collect(),
    }
}

#[test]
fn stale_sequence_does_not_panic() {
    let shrunk = sequence_route_plan(&sample_plan(3));
    let p2 = sample_plan(2);
    assert!(evaluate_route_progress(&p2, &shrunk, 1, 0.0, 0.0, 10.0, 0.0, 50.0).is_none());
    assert!(evaluate_route_progress(&p2, &shrunk, 0, 0.0, 0.0, 10.0, 0.0, 50.0).is_none());

    let grown = sequence_route_plan(&sample_plan(2));
    let p3 = sample_plan(3);
    assert!(evaluate_route_progress(&p3, &grown, 0, 0.0, 0.0, 10.0, 0.0, 50.0).is_none());
    let xml = generate_cot_route_xml(&p3, &grown);
    assert!(xml.contains("b-m-r") && xml.contains(r#"callsign="W2""#));
}

#[test]
fn corrupted_leg_indices_do_not_panic() {
    let p = sample_plan(2);
    let mut seq = sequence_route_plan(&p);
    seq.legs[0].to_index = 99;
    assert!(evaluate_route_progress(&p, &seq, 0, 0.0, 0.0, 10.0, 0.0, 50.0).is_none());
    assert!(generate_cot_route_xml(&p, &seq).contains("b-m-r"));
}

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
