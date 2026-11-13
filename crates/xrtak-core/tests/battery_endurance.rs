use xrtak_core::battery::*;

fn sample_uav(rem_pct: f64, res_pct: f64, east: f64) -> UavBatteryState {
    UavBatteryState {
        callsign: "UAV-1".into(),
        battery_remaining_pct: rem_pct,
        battery_capacity_wh: 100.0,
        current_power_draw_w: 100.0,
        reserve_margin_pct: res_pct,
        cruise_airspeed_mps: 20.0,
        current_east_m: east,
        current_north_m: 0.0,
        current_up_m: 100.0,
        home_east_m: 0.0,
        home_north_m: 0.0,
        home_up_m: 0.0,
    }
}

#[test]
fn test_remaining_at_or_below_reserve_gives_bingo() {
    let wind = WindVector::default();
    let state_at = sample_uav(15.0, 15.0, 0.0);
    assert!(compute_uav_endurance(&state_at, &wind, 16).is_bingo_critical);
    let state_below = sample_uav(10.0, 15.0, 0.0);
    assert!(compute_uav_endurance(&state_below, &wind, 16).is_bingo_critical);
    let state_above = sample_uav(50.0, 15.0, 0.0);
    assert!(!compute_uav_endurance(&state_above, &wind, 16).is_bingo_critical);
}

#[test]
fn test_headwind_reduces_rth_margin() {
    let uav = sample_uav(60.0, 15.0, 1200.0);
    let calm = WindVector::default();
    let headwind = WindVector {
        east_mps: 10.0,
        north_mps: 0.0,
    };
    let sol_calm = compute_uav_endurance(&uav, &calm, 16);
    let sol_wind = compute_uav_endurance(&uav, &headwind, 16);
    assert!(sol_wind.return_home_time_sec > sol_calm.return_home_time_sec);
    assert!(sol_wind.bingo_battery_pct > sol_calm.bingo_battery_pct);
}
