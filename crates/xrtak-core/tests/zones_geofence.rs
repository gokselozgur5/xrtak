use xrtak_core::domain::{Affiliation, TacticalZone};
use xrtak_core::zones::{is_inside_tactical_zone, point_in_polygon_en};

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

#[test]
fn point_in_concave_polygon_distinguishes_concavity() {
    let l_shape = [
        (0.0, 0.0),
        (200.0, 0.0),
        (200.0, 100.0),
        (100.0, 100.0),
        (100.0, 200.0),
        (0.0, 200.0),
    ];
    assert!(point_in_polygon_en(50.0, 50.0, &l_shape));
    assert!(point_in_polygon_en(50.0, 150.0, &l_shape));
    assert!(point_in_polygon_en(150.0, 50.0, &l_shape));
    assert!(!point_in_polygon_en(150.0, 150.0, &l_shape));
    assert!(!point_in_polygon_en(250.0, 50.0, &l_shape));
    assert!(!point_in_polygon_en(-10.0, 50.0, &l_shape));
}

#[test]
fn tactical_zone_elevation_and_boundary_checks() {
    let zone = TacticalZone {
        uid: "ZONE-1".to_string(),
        name: "Test ROZ".to_string(),
        affiliation: Affiliation::Hostile,
        vertices_en: vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
        floor_m: 50.0,
        ceiling_m: 250.0,
        inside_uids: Vec::new(),
    };
    assert!(is_inside_tactical_zone(&zone, (50.0, 100.0, 50.0)));
    assert!(!is_inside_tactical_zone(&zone, (50.0, 40.0, 50.0)));
    assert!(!is_inside_tactical_zone(&zone, (50.0, 260.0, 50.0)));
    assert!(!is_inside_tactical_zone(&zone, (150.0, 100.0, 50.0)));
}
