use xrtak_core::grid_convergence::{calculate_grid_convergence, GridSystem};

#[test]
fn latitude_between_80n_and_84n_uses_utm() {
    // Alert, Nunavut (82.5018 N, 62.3481 W): MGRS band X, UTM zone 20, central meridian 63 W.
    let sol = calculate_grid_convergence(82.5018, -62.3481).expect("valid position");
    assert_eq!(
        sol.system,
        GridSystem::Utm {
            zone: 20,
            north: true
        }
    );
    // atan(tan(0.6519 deg) * sin(82.5018 deg)) = 0.6463 deg
    assert!(
        (sol.convergence_deg - 0.6463).abs() < 1e-3,
        "convergence {}",
        sol.convergence_deg
    );
}
