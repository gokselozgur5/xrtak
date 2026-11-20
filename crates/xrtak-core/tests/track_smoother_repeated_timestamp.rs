use xrtak_core::track_smoother::{TrackObservation, TrackSmoother};

#[test]
fn repeated_timestamp_does_not_create_a_speed_spike() {
    let mut smoother = TrackSmoother::default();
    smoother.update(&TrackObservation::new(0.0, 0.0, 0.0, 0.0));
    smoother.update(&TrackObservation::new(1.0, 0.0, 0.0, 0.0));

    // Second report for the same second, 0.5 m of position jitter.
    let state = smoother.update(&TrackObservation::new(1.0, 0.5, 0.0, 0.0));

    assert!(
        state.speed_mps < 1.0,
        "stationary track reported at {} m/s",
        state.speed_mps
    );
}
