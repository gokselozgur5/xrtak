use xrtak_core::simplify_polyline_rdp;

#[test]
fn rdp_handles_long_decaying_zigzag_without_stack_overflow() {
    let n = 20_000usize;
    let pts: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            (i as f64 * 10.0, sign * (n - i) as f64 * 10.0)
        })
        .collect();
    let out = simplify_polyline_rdp(&pts, 5.0);
    assert!(out.len() <= n);
}
