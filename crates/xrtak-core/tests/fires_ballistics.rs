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

    for munition in [
        MunitionType::Artillery155mm,
        MunitionType::Mortar120mm,
        MunitionType::LoiteringMunition,
        MunitionType::TacticalRocket,
    ] {
        let sol = compute_ballistic_trajectory(origin, target, munition, None, &terrain, 32);
        assert!(sol.time_of_flight_sec > 0.0);
        assert!(sol.ground_range_m > 0.0);
    }
}
