use xrtak_core::zones::point_in_polygon_en;

#[test]
fn polygon_fewer_than_three_vertices_returns_false() {
    assert!(!point_in_polygon_en(0.0, 0.0, &[]));
    assert!(!point_in_polygon_en(0.0, 0.0, &[(10.0, 10.0)]));
    assert!(!point_in_polygon_en(0.0, 0.0, &[(0.0, 0.0), (10.0, 10.0)]));
}

#[test]
fn point_in_convex_polygon_inside_and_outside() {
    let square = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
    assert!(point_in_polygon_en(50.0, 50.0, &square));
    assert!(!point_in_polygon_en(150.0, 50.0, &square));
    assert!(!point_in_polygon_en(-10.0, 50.0, &square));
    assert!(!point_in_polygon_en(50.0, 150.0, &square));
}
