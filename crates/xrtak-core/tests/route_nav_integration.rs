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
