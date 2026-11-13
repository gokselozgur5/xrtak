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
