use xrtak_core::battery::{compute_uav_endurance, UavBatteryState, WindVector};

#[test]
fn downwind_vertex_allows_return_within_usable_time() {
    let state = UavBatteryState {
        callsign: "UAV-1".into(),
        battery_remaining_pct: 80.0,
        battery_capacity_wh: 100.0,
        current_power_draw_w: 100.0,
        reserve_margin_pct: 15.0,
        cruise_airspeed_mps: 18.0,
        current_east_m: 0.0,
        current_north_m: 0.0,
        current_up_m: 100.0,
        home_east_m: 0.0,
        home_north_m: 0.0,
        home_up_m: 0.0,
    };
    let wind = WindVector { east_mps: 10.0, north_mps: 0.0 };
    let sol = compute_uav_endurance(&state, &wind, 16);
    let r = sol.rth_polygon[0].max_radius_m;
    let round_trip_sec = r / (18.0 + 10.0) + r / (18.0 - 10.0);
    assert!(
        round_trip_sec <= sol.remaining_flight_sec + 1.0,
        "round trip {round_trip_sec:.0} s exceeds usable {} s",
        sol.remaining_flight_sec
    );
}
