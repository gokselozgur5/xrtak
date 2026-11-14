use xrtak_core::tracker::TrackPredictInput;

#[test]
fn test_track_predict_input_construction() {
    let horizons = [15.0, 30.0, 60.0];
    let input = TrackPredictInput::new(
        "UAV-01",
        "FALCON",
        (0.0, 0.0, 100.0),
        20.0,
        45.0,
        true,
        5.0,
        &horizons,
    );
    assert_eq!(input.uid, "UAV-01");
    assert_eq!(input.callsign, "FALCON");
    assert_eq!(input.speed_mps, 20.0);
    assert!(input.is_stale);
}
