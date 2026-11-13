use xrtak_core::tracker::*;

#[test]
fn test_kinematic_extrapolation_four_headings() {
    let origin = (100.0, 200.0, 50.0);
    let speed = 20.0;
    let dt = 5.0;

    let north = compute_kinematic_extrapolation(origin, speed, 0.0, 0.0, dt);
    assert!((north.0 - 100.0).abs() < 1e-9);
    assert!((north.1 - 300.0).abs() < 1e-9);

    let east = compute_kinematic_extrapolation(origin, speed, 90.0, 0.0, dt);
    assert!((east.0 - 200.0).abs() < 1e-9);
    assert!((east.1 - 200.0).abs() < 1e-9);

    let south = compute_kinematic_extrapolation(origin, speed, 180.0, 0.0, dt);
    assert!((south.0 - 100.0).abs() < 1e-9);
    assert!((south.1 - 100.0).abs() < 1e-9);

    let west = compute_kinematic_extrapolation(origin, speed, 270.0, 0.0, dt);
    assert!((west.0 - 0.0).abs() < 1e-9);
    assert!((west.1 - 200.0).abs() < 1e-9);
}
