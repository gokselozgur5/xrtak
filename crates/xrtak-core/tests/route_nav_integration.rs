use xrtak_core::route_nav::{NavWaypoint, TacticalRoutePlan, VehicleMovementProfile, WaypointType};

#[test]
fn test_integration_tactical_route_pipeline() {
    let profile = VehicleMovementProfile::new(20.0, 5.0, 5.0);
    let mut waypoints = Vec::new();
    for i in 0..5 {
        let lat = 34.0 + (i as f64) * 0.01;
        let lon = -118.0 + (i as f64) * 0.01;
        let typ = if i == 0 {
            WaypointType::Start
        } else if i == 4 {
            WaypointType::End
        } else {
            WaypointType::TurnPoint
        };
        waypoints.push(NavWaypoint::new(
            format!("WP-{}", i),
            format!("PT-{}", i),
            lat,
            lon,
            100.0,
            typ,
        ));
    }
    let plan = TacticalRoutePlan::build("RTE-TAC-1", "Alpha Convoy Route", waypoints, &profile);
    assert_eq!(plan.legs.len(), 4);
}

#[test]
fn test_integration_tracker_progression() {
    let p = VehicleMovementProfile::default();
    let w1 = NavWaypoint::new("WP-A", "ALPHA", 0.0, 0.0, 0.0, WaypointType::Start);
    let w2 = NavWaypoint::new("WP-B", "BRAVO", 0.0, 0.02, 0.0, WaypointType::TurnPoint);
    let w3 = NavWaypoint::new("WP-C", "CHARLIE", 0.0, 0.04, 0.0, WaypointType::End);
    let plan = TacticalRoutePlan::build("RTE-INT", "Test", vec![w1, w2, w3], &p);
    let mut tracker = xrtak_core::route_nav::RouteNavTracker::new(plan);
    let st1 = tracker.evaluate_position(0.0, 0.01, 0.0, 15.0, Some(90.0));
    assert_eq!(st1.unwrap().target_waypoint_id, "WP-B");
    tracker.advance_if_reached(10.0);
    assert_eq!(tracker.current_target_waypoint_id(), Some("WP-C"));
}
