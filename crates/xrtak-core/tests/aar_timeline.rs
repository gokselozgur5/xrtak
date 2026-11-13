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

#[test]
fn timeline_sample_empty_and_boundaries() {
    let tl_empty = AarTimeline::new(8, 0.1);
    assert!(tl_empty.sample_at(5.0).is_none());

    let mut tl = AarTimeline::new(8, 0.1);
    let mut map = std::collections::HashMap::new();
    let origin = xrtak_core::GeoOrigin::default();
    let xml =
        r#"<event uid="U1" type="a-f-G"><point lat="39.92" lon="32.85" hae="850.0"/></event>"#;
    let ent = xrtak_core::parse_cot_xml(xml, &origin, 1000).expect("valid entity");
    map.insert("U1".to_string(), ent);
    tl.maybe_record(10.0, 10_000, &map);
    tl.maybe_record(20.0, 20_000, &map);

    let before = tl.sample_at(5.0).expect("frame before");
    assert_eq!(before.sim_time_sec, 10.0);
    let after = tl.sample_at(25.0).expect("frame after");
    assert_eq!(after.sim_time_sec, 20.0);
}
