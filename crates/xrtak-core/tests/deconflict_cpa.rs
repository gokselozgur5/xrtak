use xrtak_core::deconflict::compute_pairwise_cpa;

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
