use xrtak_core::aar::lerp_angle_deg;

#[test]
fn test_lerp_angle_endpoints() {
    assert_eq!(lerp_angle_deg(45.0, 90.0, 0.0), 45.0);
    assert_eq!(lerp_angle_deg(45.0, 90.0, 1.0), 90.0);
    assert_eq!(lerp_angle_deg(350.0, 10.0, 0.0), 350.0);
    assert_eq!(lerp_angle_deg(350.0, 10.0, 1.0), 10.0);
}

#[test]
fn test_lerp_angle_shortest_arc_wrap() {
    let mid = lerp_angle_deg(350.0, 10.0, 0.5);
    assert_eq!(mid, 0.0);

    let quarter = lerp_angle_deg(350.0, 10.0, 0.25);
    assert_eq!(quarter, 355.0);

    let three_quarters = lerp_angle_deg(350.0, 10.0, 0.75);
    assert_eq!(three_quarters, 5.0);
}
