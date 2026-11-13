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

struct SteepRamp;
impl ElevationProvider for SteepRamp {
    fn sample_relief_m(&self, east: f64, _n: f64) -> f64 {
        east * 0.5
    }
}

#[test]
fn steep_ramp_terrain_is_unsuitable() {
    let suitability = evaluate_hlz_suitability(100.0, 0.0, 25.0, &SteepRamp, &[]);
    assert!(suitability.max_slope_deg > 8.0);
    assert!(!suitability.is_slope_acceptable);
    assert_eq!(suitability.overall_suitability, "AMBER - STEEP TERRAIN");
}

#[test]
fn point_inside_threat_dome_is_unsuitable() {
    use xrtak_core::domain::{Affiliation, ThreatDome};

    let dome = ThreatDome {
        uid: "DOME-1".into(),
        callsign: "SAM-BATTERY".into(),
        affiliation: Affiliation::Hostile,
        east_m: 100.0,
        up_m: 0.0,
        north_m: 100.0,
        radius_m: 500.0,
        ceiling_m: 2000.0,
        engaged_uids: Vec::new(),
    };
    let suitability = evaluate_hlz_suitability(120.0, 120.0, 25.0, &Flat, &[dome]);
    assert!(!suitability.is_threat_clear);
    assert!(suitability.min_threat_distance_m < 0.0);
    assert_eq!(suitability.overall_suitability, "AMBER - THREAT PROXIMITY");
}
