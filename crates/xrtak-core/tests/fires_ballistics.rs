use xrtak_core::fires::*;
use xrtak_core::geo::ElevationProvider;

struct FlatTerrain;
impl ElevationProvider for FlatTerrain {
    fn sample_relief_m(&self, _east: f64, _north: f64) -> f64 {
        0.0
    }
}

struct HighRidge {
    elevation: f64,
}
impl ElevationProvider for HighRidge {
    fn sample_relief_m(&self, _east: f64, _north: f64) -> f64 {
        self.elevation
    }
}

#[test]
fn test_ballistic_time_of_flight_is_positive() {
    let origin = (0.0, 0.0, 10.0);
    let target = (1500.0, 500.0, 10.0);
    let terrain = FlatTerrain;

    let expected = [
        (MunitionType::Artillery155mm, 66.3, 1500.0),
        (MunitionType::Mortar120mm, 41.9, 1500.0),
        (MunitionType::LoiteringMunition, 2.0, 1500.0),
        (MunitionType::TacticalRocket, 94.8, 1500.0),
    ];

    for (munition, expected_tof, expected_range) in expected {
        let sol = compute_ballistic_trajectory(origin, target, munition, None, &terrain, 32);
        assert_eq!(sol.time_of_flight_sec, expected_tof);
        assert_eq!(sol.ground_range_m, expected_range);
    }
}

#[test]
fn test_flat_terrain_apogee_within_one_percent() {
    let origin = (0.0, 0.0, 0.0);
    let munition = MunitionType::Mortar120mm;
    let angle_deg: f64 = 45.0;
    let terrain = FlatTerrain;

    let v0 = munition.nominal_velocity_mps();
    let theta: f64 = angle_deg.to_radians();
    let g: f64 = 9.80665;
    let natural_range = v0.powi(2) * (2.0 * theta).sin() / g;
    let target = (natural_range, 0.0, 0.0);

    let sol =
        compute_ballistic_trajectory(origin, target, munition, Some(angle_deg), &terrain, 1000);

    let expected_apogee = (v0 * theta.sin()).powi(2) / (2.0 * g);
    let diff_ratio = (sol.max_ordinate_m - expected_apogee).abs() / expected_apogee;
    assert!(diff_ratio < 0.01);
}

#[test]
fn test_ridge_triggers_crest_clearance_failure() {
    let origin = (0.0, 0.0, 0.0);
    let target = (2000.0, 0.0, 0.0);
    let terrain = HighRidge { elevation: 3000.0 };

    let sol = compute_ballistic_trajectory(
        origin,
        target,
        MunitionType::Mortar120mm,
        Some(45.0),
        &terrain,
        64,
    );

    assert!(!sol.is_crest_cleared);
    assert!(sol.min_crest_clearance_m < 0.0);
}
