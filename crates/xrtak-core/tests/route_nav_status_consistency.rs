use xrtak_core::route_nav::{
    NavWaypoint, RouteNavTracker, TacticalRoutePlan, VehicleMovementProfile, WaypointType,
};

#[test]
fn status_is_consistent_when_a_waypoint_is_reached() {
    let profile = VehicleMovementProfile::default();
    let wps = vec![
        NavWaypoint::new("A", "A", 0.0, 0.00, 0.0, WaypointType::Start),
        NavWaypoint::new("B", "B", 0.0, 0.02, 0.0, WaypointType::TurnPoint),
        NavWaypoint::new("C", "C", 0.0, 0.04, 0.0, WaypointType::End),
    ];
    let mut tracker = RouteNavTracker::new(TacticalRoutePlan::build("R", "r", wps, &profile));

    // 11 m short of B: inside the 25 m acceptance radius, so the tracker switches to leg 1.
    let status = tracker
        .evaluate_position(0.0, 0.0199, 0.0, 15.0, None)
        .unwrap();
    let leg = &tracker.route.legs[status.current_leg_index];
    assert_eq!(status.target_waypoint_id, leg.to_wp);
}
