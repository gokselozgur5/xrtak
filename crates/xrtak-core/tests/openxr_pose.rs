use xrtak_core::openxr_pose::{
    OpenXrPoseAdapter, OpenXrPoseSample, PoseOrientation, PosePosition, PoseTrackingFlags,
};

#[test]
fn test_integration_openxr_pose_transformation_and_gaze() {
    let adapter = OpenXrPoseAdapter::new("OP-TEST", "EAGLE-1", "Quest 3").with_table_transform(
        (50.0, 2.0, 100.0),
        90.0,
        2.0,
    );

    // Non-identity orientation: 90 deg rotation around Y axis
    let non_identity_quat =
        PoseOrientation::new(0.0, 2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);

    let sample = OpenXrPoseSample {
        position: PosePosition::new(1.0, 0.5, -2.0),
        orientation: non_identity_quat,
        tracking_flags: PoseTrackingFlags::FULLY_TRACKED,
        timestamp_ns: 120_000_000,
    };

    let tactical_pose = adapter.transform_pose(&sample);
    assert!(tactical_pose.is_valid);
    assert_eq!(tactical_pose.timestamp_unix_ms, 120);

    // With Y-rotation of 90 deg and table yaw 90 deg, compass yaw is 0 deg (North)
    assert!(tactical_pose.yaw_deg < 1e-4 || (tactical_pose.yaw_deg - 360.0).abs() < 1e-4);
    assert!((tactical_pose.forward_enu.2 - 1.0).abs() < 1e-4);

    // Gaze projection test
    let ground_hit = adapter.project_gaze_to_ground(&tactical_pose, 0.0);
    // Forward direction has up = 0, so should return None for ground hit
    assert!(ground_hit.is_none());
}
