use xrtak_core::deconflict::{compute_pairwise_cpa, CpaRiskLevel};

#[test]
fn test_parallel_equal_velocity_tracks_give_zero_t_cpa() {
    let p_a = (0.0, 0.0, 100.0);
    let p_b = (100.0, 0.0, 100.0);
    let rep = compute_pairwise_cpa("A", p_a, 20.0, 0.0, "B", p_b, 20.0, 0.0);
    assert_eq!(rep.t_cpa_sec, 0.0);
    assert_eq!(rep.horiz_separation_m, 100.0);
    assert_eq!(rep.vert_separation_m, 0.0);
}

#[test]
fn test_head_on_tracks_analytic_t_cpa() {
    let p_a = (0.0, 0.0, 100.0);
    let p_b = (0.0, 1000.0, 100.0);
    let rep = compute_pairwise_cpa("A", p_a, 20.0, 0.0, "B", p_b, 20.0, 180.0);
    assert_eq!(rep.t_cpa_sec, 25.0);
    assert_eq!(rep.horiz_separation_m, 0.0);
    assert_eq!(rep.vert_separation_m, 0.0);
}

#[test]
fn test_diverging_tracks_negative_t_cpa() {
    let p_a = (0.0, 0.0, 100.0);
    let p_b = (0.0, 1000.0, 100.0);
    let rep = compute_pairwise_cpa("A", p_a, 20.0, 180.0, "B", p_b, 20.0, 0.0);
    assert!(rep.t_cpa_sec <= 0.0);
}

#[test]
fn test_critical_and_advisory_risk_levels() {
    let p0 = (0.0, 0.0, 0.0);
    let crit = compute_pairwise_cpa("A", p0, 10.0, 0.0, "B", (0.0, 200.0, 0.0), 10.0, 180.0);
    assert_eq!(crit.risk_level, CpaRiskLevel::Critical);

    let adv = compute_pairwise_cpa("A", p0, 5.0, 0.0, "B", (400.0, 800.0, 50.0), 5.0, 180.0);
    assert_eq!(adv.risk_level, CpaRiskLevel::Advisory);
}

#[test]
fn test_clear_risk_level() {
    let p0 = (0.0, 0.0, 0.0);
    let clear = compute_pairwise_cpa("A", p0, 10.0, 0.0, "B", (5000.0, 0.0, 0.0), 10.0, 0.0);
    assert_eq!(clear.risk_level, CpaRiskLevel::Clear);
}
