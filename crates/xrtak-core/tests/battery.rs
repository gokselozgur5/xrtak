use xrtak_core::battery::*;

#[test]
fn test_altitude_counts_towards_rth_distance_and_bingo() {
    let state = UavBatteryState {
        callsign: "VALKYRIE-ALT".into(),
        battery_remaining_pct: 16.0,
        battery_capacity_wh: 100.0,
        current_power_draw_w: 200.0,
        reserve_margin_pct: 15.0,
        cruise_airspeed_mps: 15.0,
        current_east_m: 0.0,
        current_north_m: 0.0,
        current_up_m: 500.0,
        home_east_m: 0.0,
        home_north_m: 0.0,
        home_up_m: 0.0,
    };
    let sol = compute_uav_endurance(&state, &WindVector::default(), 16);
    assert_eq!(sol.distance_to_home_m, 500.0);
    assert!(sol.return_home_time_sec > 0.0);
    assert!(sol.is_bingo_critical);
}
