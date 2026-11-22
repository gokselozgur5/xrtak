use xrtak_core::track_simplify::{simplify_track_with_config, TrackPoint, TrackSimplifyConfig};

#[test]
fn test_track_simplify_max_error_retained_split_point() {
    let a = TrackPoint::new(0.0, 0.0, 0.0);
    let b = TrackPoint::new(0.001, 0.001, 0.0);
    let c = TrackPoint::new(0.0, 0.002, 0.0);
    let config = TrackSimplifyConfig {
        epsilon_m: 5.0,
        include_elevation: false,
        max_time_gap_sec: None,
    };
    let result = simplify_track_with_config(&[a, b, c], &config);
    assert_eq!(result.simplified_count, 3);
    assert_eq!(result.max_error_m, 0.0);
}
