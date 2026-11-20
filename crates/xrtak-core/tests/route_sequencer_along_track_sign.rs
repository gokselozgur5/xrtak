use xrtak_core::route_sequencer::along_track_distance_m;

#[test]
fn point_behind_the_leg_start_has_no_positive_along_track_distance() {
    let behind = along_track_distance_m(0.0, -0.01, 0.0, 0.0, 0.0, 1.0);
    let ahead = along_track_distance_m(0.0, 0.01, 0.0, 0.0, 0.0, 1.0);

    assert!((ahead - 1111.95).abs() < 0.1, "ahead = {ahead}");
    assert!(behind <= 0.0, "behind = {behind}");
}
