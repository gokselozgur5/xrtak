use xrtak_core::geo::{
    compute_elevation_profile, ProceduralReliefModel, MAX_ELEVATION_PROFILE_SAMPLES,
    MAX_ELEVATION_PROFILE_WAYPOINTS,
};

#[test]
fn compute_elevation_profile_bounds_waypoints_and_total_samples() {
    let oversized: Vec<(f64, f64)> = (0..5_000).map(|i| (i as f64 * 10.0, 0.0)).collect();
    let capped = compute_elevation_profile(&ProceduralReliefModel, &oversized, 100.0, 28);
    let exact = compute_elevation_profile(
        &ProceduralReliefModel,
        &oversized[..MAX_ELEVATION_PROFILE_WAYPOINTS],
        100.0,
        28,
    );
    assert_eq!(
        capped.samples.len(),
        (MAX_ELEVATION_PROFILE_WAYPOINTS - 1) * 28 + 1
    );
    assert_eq!(capped.total_distance_m, exact.total_distance_m);
    assert!(capped.samples.len() <= MAX_ELEVATION_PROFILE_SAMPLES);

    let dense = compute_elevation_profile(
        &ProceduralReliefModel,
        &[(0.0, 0.0), (100.0, 0.0)],
        0.0,
        usize::MAX,
    );
    assert_eq!(dense.samples.len(), MAX_ELEVATION_PROFILE_SAMPLES);
}
