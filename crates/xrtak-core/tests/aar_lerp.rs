use xrtak_core::aar::lerp_angle_deg;

#[test]
fn test_lerp_angle_endpoints() {
    assert_eq!(lerp_angle_deg(45.0, 90.0, 0.0), 45.0);
    assert_eq!(lerp_angle_deg(45.0, 90.0, 1.0), 90.0);
    assert_eq!(lerp_angle_deg(350.0, 10.0, 0.0), 350.0);
    assert_eq!(lerp_angle_deg(350.0, 10.0, 1.0), 10.0);
}
