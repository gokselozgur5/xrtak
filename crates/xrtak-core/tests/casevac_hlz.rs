use xrtak_core::casevac::evaluate_hlz_suitability;
use xrtak_core::geo::ElevationProvider;

struct Flat;
impl ElevationProvider for Flat {
    fn sample_relief_m(&self, _e: f64, _n: f64) -> f64 {
        100.0
    }
}

#[test]
fn flat_ground_with_no_threats_is_suitable() {
    let suitability = evaluate_hlz_suitability(0.0, 0.0, 25.0, &Flat, &[]);
    assert_eq!(suitability.max_slope_deg, 0.0);
    assert!(suitability.is_slope_acceptable);
    assert!(suitability.is_threat_clear);
    assert_eq!(suitability.overall_suitability, "GREEN - SUITABLE");
}
