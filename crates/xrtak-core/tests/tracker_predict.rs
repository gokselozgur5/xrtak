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

#[test]
fn test_cep_growth_monotonic_with_floor() {
    let speed = 25.0;

    let cep_floor = compute_cep_radius_m(0.5, speed, 0.0);
    assert!((cep_floor - 2.0).abs() < 1e-9);

    let mut last_cep = cep_floor;
    for &dt in &[1.0, 5.0, 10.0, 30.0, 60.0] {
        let cep = compute_cep_radius_m(1.0, speed, dt);
        assert!(cep > last_cep);
        last_cep = cep;
    }
}
