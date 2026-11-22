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

#[test]
fn test_track_predict_input_predict_method() {
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
    let pred = input.predict();
    assert_eq!(pred.uid, "UAV-01");
    assert_eq!(pred.points.len(), 3);
    assert_eq!(pred.points[0].east_m, 212.13);
    assert_eq!(pred.points[0].north_m, 212.13);
    assert_eq!(pred.points[0].up_m, 100.0);
    assert_eq!(pred.points[1].east_m, 424.26);
    assert_eq!(pred.points[1].north_m, 424.26);
    assert_eq!(pred.points[2].east_m, 848.53);
    assert_eq!(pred.points[2].north_m, 848.53);
}
