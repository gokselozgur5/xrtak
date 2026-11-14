use xrtak_core::deconflict::CpaTrack;

#[test]
fn test_cpa_track_construction() {
    let track = CpaTrack::new("UAV-1", (0.0, 0.0, 100.0), 25.0, 90.0);
    assert_eq!(track.uid, "UAV-1");
    assert_eq!(track.speed_mps, 25.0);
}
