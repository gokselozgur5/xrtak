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
