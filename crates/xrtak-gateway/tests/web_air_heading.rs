const ROOT_INDEX: &str = include_str!("../../../index.html");

#[test]
fn air_track_icon_heading_points_along_reported_course() {
    let air_block = ROOT_INDEX
        .split("ent.dimension === 'air'")
        .nth(1)
        .expect("ent.dimension === 'air' branch must exist in index.html")
        .split('}')
        .next()
        .expect("air block extract");

    let is_neg_pi_2 = air_block.contains("rotateX(-Math.PI / 2)");
    let is_pos_pi_2 = air_block.contains("rotateX(Math.PI / 2)");

    assert!(
        is_neg_pi_2 || is_pos_pi_2,
        "air icon geometry must apply rotateX"
    );

    let angle = if is_neg_pi_2 {
        -std::f64::consts::FRAC_PI_2
    } else {
        std::f64::consts::FRAC_PI_2
    };

    // Three.js ConeGeometry apex starts at (0, +height/2, 0).
    let h = 0.042_f64;
    let apex_y = h / 2.0;

    // After rotateX(angle):
    // y' = y * cos(angle)
    // z' = y * sin(angle)
    let z_prime = apex_y * angle.sin();

    // After yaw: rotation.y = -course_rad
    // x_final = z_prime * sin(yaw) = -z_prime * sin(course_rad)
    // z_final = z_prime * cos(yaw) = z_prime * cos(course_rad)
    // In table coordinates: North is -Z (z < 0), East is +X (x > 0).

    // Course 0 deg (North): must have z_final < 0 (pointing North)
    let c0_rad = 0.0_f64.to_radians();
    let c0_z = z_prime * c0_rad.cos();
    assert!(
        c0_z < -1e-6,
        "course 0 deg must point North (z < 0), but got z = {c0_z}"
    );

    // Course 90 deg (East): must have x_final > 0 (pointing East)
    let c90_rad = 90.0_f64.to_radians();
    let c90_x = -z_prime * c90_rad.sin();
    assert!(
        c90_x > 1e-6,
        "course 90 deg must point East (x > 0), but got x = {c90_x}"
    );

    // Course 225 deg (South-West): must have x_final < 0 (West) and z_final > 0 (South)
    let c225_rad = 225.0_f64.to_radians();
    let c225_x = -z_prime * c225_rad.sin();
    let c225_z = z_prime * c225_rad.cos();
    assert!(
        c225_x < -1e-6 && c225_z > 1e-6,
        "course 225 deg must point South-West (x < 0, z > 0), but got x = {c225_x}, z = {c225_z}"
    );
}
