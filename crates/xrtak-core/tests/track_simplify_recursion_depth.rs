use xrtak_core::track_simplify::{simplify_track_rdp, TrackPoint};

#[test]
fn long_zigzag_track_does_not_overflow_the_stack() {
    let n = 5000usize;
    let points: Vec<TrackPoint> = (0..n)
        .map(|i| {
            let amp = 0.01 * (n - i) as f64 / n as f64;
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            TrackPoint::new(39.0 + sign * amp, 32.0 + i as f64 * 1e-4, 0.0)
        })
        .collect();
    assert!(simplify_track_rdp(&points, 1.0).len() >= n - 1);
}

#[test]
fn negative_epsilon_terminates() {
    let p = TrackPoint::new(39.0, 32.0, 0.0);
    let out = simplify_track_rdp(&[p.clone(), p.clone(), p], -1.0);
    assert!(out.len() <= 3);
}
