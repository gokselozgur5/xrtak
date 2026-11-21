use xrtak_core::deconflict::CpaTrack;

#[test]
fn test_cpa_track_construction() {
    let track = CpaTrack::new("UAV-1", (0.0, 0.0, 100.0), 25.0, 90.0);
    assert_eq!(track.uid, "UAV-1");
    assert_eq!(track.speed_mps, 25.0);
}

#[test]
fn test_cpa_track_compute_cpa_method() {
    let t1 = CpaTrack::new("A", (0.0, 0.0, 100.0), 20.0, 0.0);
    let t2 = CpaTrack::new("B", (0.0, 600.0, 100.0), 20.0, 180.0);
    let report = t1.compute_cpa(&t2);
    assert_eq!(
        report.risk_level,
        xrtak_core::deconflict::CpaRiskLevel::Critical
    );
}
