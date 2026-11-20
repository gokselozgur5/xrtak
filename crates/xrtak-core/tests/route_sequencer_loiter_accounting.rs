use xrtak_core::route_sequencer::{
    evaluate_route_progress, sequence_route_plan, RoutePlan, RouteWaypoint,
};

#[test]
fn loiter_is_counted_after_arrival_and_in_the_remaining_time() {
    let plan = RoutePlan {
        route_uid: "R".into(),
        callsign: "C".into(),
        default_speed_mps: 10.0,
        start_time_epoch_sec: 1000.0,
        waypoints: vec![
            RouteWaypoint::new("A", 0.0, 0.0, 0.0),
            RouteWaypoint::new("B", 0.0, 0.1, 0.0).with_loiter(600.0),
        ],
    };
    let seq = sequence_route_plan(&plan);
    assert!((seq.total_duration_sec - 1711.9).abs() < 0.2);

    let at_start = evaluate_route_progress(&plan, &seq, 0, 0.0, 0.0, 10.0, 1000.0, 50.0).unwrap();
    assert!((at_start.remaining_route_time_sec - seq.total_duration_sec).abs() < 0.2);
    assert!((seq.legs[0].eta_epoch_sec - 2111.9).abs() < 0.2);
}
