use xrtak_core::{compute_link_los, ElevationProvider};

struct RidgeElevation;

impl ElevationProvider for RidgeElevation {
    fn sample_relief_m(&self, east_m: f64, _north_m: f64) -> f64 {
        if east_m.abs() < 50.0 {
            100.0
        } else {
            10.0
        }
    }
}

#[test]
fn test_link_los_blocked_by_ridge() {
    let res = compute_link_los(&RidgeElevation, (-100.0, 30.0, 0.0), (100.0, 30.0, 0.0), 20);
    assert!(!res.clear);
    assert!(res.min_clearance_m < 0.0);
    assert!(res.obstruction_enu.is_some());
}

#[test]
fn test_link_los_clear_above_ridge() {
    let res = compute_link_los(
        &RidgeElevation,
        (-100.0, 150.0, 0.0),
        (100.0, 150.0, 0.0),
        20,
    );
    assert!(res.clear);
    assert!(res.min_clearance_m >= 50.0);
    assert!(res.obstruction_enu.is_none());
    assert!((res.slant_range_m - 200.0).abs() < 1e-6);
}

#[test]
fn test_link_los_zero_length() {
    let point = (0.0, 150.0, 0.0);
    let res = compute_link_los(&RidgeElevation, point, point, 10);
    assert_eq!(res.from_enu, point);
    assert_eq!(res.to_enu, point);
    assert_eq!(res.slant_range_m, 0.0);
    assert!(res.clear);
}
