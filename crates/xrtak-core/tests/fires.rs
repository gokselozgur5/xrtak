use xrtak_core::fires::{compute_ballistic_trajectory, MunitionType};
use xrtak_core::geo::ElevationProvider;

struct Flat;
impl ElevationProvider for Flat {
    fn sample_relief_m(&self, _e: f64, _n: f64) -> f64 {
        0.0
    }
}

#[test]
fn max_ordinate_matches_muzzle_velocity_and_elevation() {
    let sol = compute_ballistic_trajectory(
        (0.0, 0.0, 0.0),
        (2000.0, 0.0, 0.0),
        MunitionType::Mortar120mm,
        Some(60.0),
        &Flat,
        2000,
    );
    let (v0, g, th) = (240.0_f64, 9.80665_f64, 60.0_f64.to_radians());
    let apex = (v0 * th.sin()).powi(2) / (2.0 * g);
    assert!(
        (sol.max_ordinate_m - apex).abs() < 0.01 * apex,
        "max ordinate {} m, expected {apex:.1} m",
        sol.max_ordinate_m
    );
}
