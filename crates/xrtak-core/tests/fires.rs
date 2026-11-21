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

#[test]
fn test_repro_fires_enu_ground_range() {
    let payload = serde_json::json!({
        "origin_enu": [-750.0, 60.0, -600.0],
        "target_enu": [650.0, 95.0, 700.0],
        "munition": "artillery",
        "elevation_angle_deg": 42.0,
        "samples": 36
    });
    let origin: (f64, f64, f64) = serde_json::from_value(payload["origin_enu"].clone()).unwrap();
    let target: (f64, f64, f64) = serde_json::from_value(payload["target_enu"].clone()).unwrap();
    let sol = compute_ballistic_trajectory(
        origin,
        target,
        MunitionType::Artillery155mm,
        Some(42.0),
        &Flat,
        36,
    );
    assert_eq!(sol.ground_range_m, 1910.5);
}
