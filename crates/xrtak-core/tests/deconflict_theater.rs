use xrtak_core::deconflict::*;

#[test]
fn theater_pair_count_matches_combination_formula() {
    let empty: [(&str, (f64, f64, f64), f64, f64); 0] = [];
    let single = [("U1", (0.0, 0.0, 100.0), 20.0, 0.0)];
    let pair = [
        ("U1", (0.0, 0.0, 100.0), 20.0, 0.0),
        ("U2", (0.0, 1000.0, 100.0), 20.0, 180.0),
    ];
    let count = |arr: &[(&str, (f64, f64, f64), f64, f64)]| {
        evaluate_theater_deconfliction(arr).total_evaluated_pairs
    };
    assert_eq!(count(&empty), 0);
    assert_eq!(count(&single), 0);
    assert_eq!(count(&pair), 1);
}

#[test]
fn theater_max_risk_aggregation() {
    let clear = [
        ("U1", (0.0, 0.0, 100.0), 20.0, 270.0),
        ("U2", (1000.0, 0.0, 100.0), 20.0, 90.0),
    ];
    let res_clear = evaluate_theater_deconfliction(&clear);
    assert_eq!(res_clear.max_risk, CpaRiskLevel::Clear);
    assert!(res_clear.conflicts.is_empty());

    let mixed = [
        ("U1", (0.0, 0.0, 100.0), 20.0, 0.0),
        ("U2", (0.0, 500.0, 100.0), 20.0, 180.0),
        ("U3", (5000.0, 5000.0, 500.0), 15.0, 90.0),
    ];
    let res_mixed = evaluate_theater_deconfliction(&mixed);
    assert_eq!(res_mixed.max_risk, CpaRiskLevel::Critical);
    assert_eq!(res_mixed.conflicts.len(), 1);
}
