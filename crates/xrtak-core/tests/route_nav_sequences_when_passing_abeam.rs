use xrtak_core::route_nav::*;

#[test]
fn route_is_sequenced_when_waypoints_are_passed_abeam() {
    let wps = vec![
        NavWaypoint::new("A", "A", 0.0, 0.00, 0.0, WaypointType::Start),
        NavWaypoint::new("B", "B", 0.0, 0.02, 0.0, WaypointType::TurnPoint),
        NavWaypoint::new("C", "C", 0.0, 0.04, 0.0, WaypointType::End),
    ];
    let plan = TacticalRoutePlan::build("R1", "r", wps, &VehicleMovementProfile::default());
    let mut tracker = RouteNavTracker::new(plan);
    let mut flagged_off_track = false;
    for i in 0..=84 {
        let lon = i as f64 * 0.0005;
        let s = tracker.evaluate_position(0.00036, lon, 0.0, 15.0, None).unwrap();
        flagged_off_track |= s.is_off_track;
    }
    assert!(!flagged_off_track);
    assert!(
        tracker.is_completed,
        "past last wp, tracker still on leg {}",
        tracker.current_leg_index
    );
}
