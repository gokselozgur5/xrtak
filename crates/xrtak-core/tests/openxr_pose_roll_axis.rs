use xrtak_core::openxr_pose::{
    OpenXrPoseAdapter, OpenXrPoseSample, PoseOrientation, PosePosition, PoseTrackingFlags,
};

fn sample(x: f64, y: f64, z: f64, w: f64) -> OpenXrPoseSample {
    OpenXrPoseSample {
        position: PosePosition::new(0.0, 1.6, 0.0),
        orientation: PoseOrientation::new(x, y, z, w),
        tracking_flags: PoseTrackingFlags::FULLY_TRACKED,
        timestamp_ns: 0,
    }
}

#[test]
fn roll_is_the_rotation_about_the_view_axis() {
    let adapter = OpenXrPoseAdapter::new("OP-1", "VIPER", "Quest 3");
    let half = 15f64.to_radians();
    let nod = adapter.transform_pose(&sample(half.sin(), 0.0, 0.0, half.cos()));
    assert!(
        (nod.pitch_deg - 30.0).abs() < 1e-6,
        "pitch {}",
        nod.pitch_deg
    );
    assert!(
        nod.roll_deg.abs() < 1e-6,
        "nod reported as roll {}",
        nod.roll_deg
    );
    let tilt = adapter.transform_pose(&sample(0.0, 0.0, half.sin(), half.cos()));
    assert!(
        (tilt.roll_deg.abs() - 30.0).abs() < 1e-6,
        "tilt roll {}",
        tilt.roll_deg
    );
}
