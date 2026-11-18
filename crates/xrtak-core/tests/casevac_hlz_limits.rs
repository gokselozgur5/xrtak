use xrtak_core::casevac::evaluate_hlz_suitability;
use xrtak_core::geo::ElevationProvider;

struct NoData;
impl ElevationProvider for NoData {
    fn sample_relief_m(&self, _e: f64, _n: f64) -> f64 {
        f64::NAN
    }
}

struct Ramp;
impl ElevationProvider for Ramp {
    fn sample_relief_m(&self, east_m: f64, _n: f64) -> f64 {
        east_m // 45 degree slope
    }
}

#[test]
fn missing_elevation_is_not_reported_as_suitable() {
    let s = evaluate_hlz_suitability(0.0, 0.0, 25.0, &NoData, &[]);
    assert_ne!(s.overall_suitability, "GREEN - SUITABLE");
}

#[test]
fn slope_of_45_degrees_is_unsuitable() {
    let s = evaluate_hlz_suitability(0.0, 0.0, 25.0, &Ramp, &[]);
    assert_eq!(s.max_slope_deg, 45.0);
    assert!(s.overall_suitability.starts_with("RED"), "{}", s.overall_suitability);
}
