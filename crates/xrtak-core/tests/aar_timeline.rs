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
