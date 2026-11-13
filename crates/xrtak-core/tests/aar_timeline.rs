use xrtak_core::aar::*;

#[test]
fn timeline_constructor_enforces_floors() {
    let tl = AarTimeline::new(2, 0.001);
    assert_eq!(tl.max_frames, 8);
    assert_eq!(tl.min_capture_interval_sec, 0.05);

    let summary = tl.summary();
    assert_eq!(summary.frame_count, 0);
    assert_eq!(summary.max_frames, 8);
    assert_eq!(summary.min_time_sec, 0.0);
    assert_eq!(summary.max_time_sec, 0.0);
}

#[test]
fn timeline_throttle_and_eviction() {
    let mut tl = AarTimeline::new(8, 0.1);
    let map = std::collections::HashMap::new();
    assert!(tl.maybe_record(1.0, 1000, &map));
    assert!(!tl.maybe_record(1.05, 1050, &map));
    assert!(tl.maybe_record(1.15, 1150, &map));

    for i in 2..9 {
        assert!(tl.maybe_record(1.0 + (i as f64) * 0.2, 1000 + i * 200, &map));
    }
    assert_eq!(tl.frames.len(), 8);
    assert_eq!(tl.frames.front().unwrap().sim_time_sec, 1.15);
    assert_eq!(tl.frames.back().unwrap().sim_time_sec, 2.6);
}
