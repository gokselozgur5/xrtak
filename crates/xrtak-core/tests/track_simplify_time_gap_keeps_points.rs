use xrtak_core::track_simplify::{simplify_track_with_config, TrackPoint, TrackSimplifyConfig};

#[test]
fn first_point_after_a_time_gap_is_kept() {
    let points: Vec<TrackPoint> = [
        (39.00, 1_000u64),
        (39.01, 2_000),
        (39.02, 50_000),
        (39.03, 51_000),
    ]
    .iter()
    .map(|&(lat, t)| TrackPoint::new(lat, 32.0, 100.0).with_time(t))
    .collect();
    let config = TrackSimplifyConfig {
        epsilon_m: 10.0,
        include_elevation: false,
        max_time_gap_sec: Some(10.0),
    };

    let result = simplify_track_with_config(&points, &config);

    let lats: Vec<f64> = result.points.iter().map(|p| p.lat).collect();
    assert_eq!(lats, vec![39.00, 39.01, 39.02, 39.03]);
}
