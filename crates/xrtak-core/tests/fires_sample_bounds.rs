use xrtak_core::battery::{compute_uav_endurance, UavBatteryState, WindVector};
use xrtak_core::fires::{compute_ballistic_trajectory, MunitionType};
use xrtak_core::geo::ProceduralReliefModel;

#[test]
fn endurance_bounds_radial_step_count() {
    let state: UavBatteryState = serde_json::from_str(
        r#"{"callsign":"U","battery_remaining_pct":80.0,"battery_capacity_wh":100.0,
        "current_power_draw_w":100.0,"reserve_margin_pct":15.0,"cruise_airspeed_mps":20.0,
        "current_east_m":0.0,"current_north_m":0.0,"current_up_m":0.0,
        "home_east_m":0.0,"home_north_m":0.0,"home_up_m":0.0}"#,
    )
    .unwrap();
    let sol = compute_uav_endurance(&state, &WindVector::default(), usize::MAX);
    assert!(sol.rth_polygon.len() <= 1_000_000);
}

#[test]
fn ballistics_bounds_sample_count() {
    let sol = compute_ballistic_trajectory(
        (0.0, 0.0, 50.0),
        (1500.0, 1500.0, 80.0),
        MunitionType::Mortar120mm,
        None,
        &ProceduralReliefModel,
        usize::MAX,
    );
    assert!(sol.trajectory.len() <= 1_000_001);
}
