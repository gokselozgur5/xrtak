use xrtak_core::track_smoother::{TrackObservation, TrackSmoother};

#[test]
fn speed_clamp_limits_position_jump() {
    let mut smoother = TrackSmoother::default();
    let s1 = smoother.update(&TrackObservation::new(0.0, 0.0, 0.0, 0.0));
    let s2 = smoother.update(&TrackObservation::new(1.0, 1000.0, 0.0, 0.0));

    let dt = s2.timestamp_sec - s1.timestamp_sec;
    let max_allowed_speed = smoother.config.max_speed_clamp_mps.unwrap();
    let implied_speed = (s2.east_m - s1.east_m).abs() / dt;

    assert!(
        implied_speed <= max_allowed_speed,
        "implied speed {implied_speed} exceeds clamp {max_allowed_speed}"
    );
}
