use xrtak_core::corridor::*;

fn shoelace_area(ring: &[(f64, f64)]) -> f64 {
    let mut s = 0.0;
    for w in ring.windows(2) {
        s += w[0].0 * w[1].1 - w[1].0 * w[0].1;
    }
    s.abs() / 2.0
}

#[test]
fn corridor_rings_are_closed() {
    let lines: [Vec<(f64, f64)>; 3] = [
        vec![(0.0, 0.0)],
        vec![(0.0, 0.0), (1000.0, 0.0)],
        vec![(0.0, 0.0), (1000.0, 0.0), (1000.0, 500.0)],
    ];
    for line in &lines {
        let ring = build_buffered_corridor_polygon(line, 50.0);
        assert!(ring.len() >= 5, "ring too short: {}", ring.len());
        assert_eq!(ring.first(), ring.last());
    }
}

#[test]
