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
fn straight_corridor_area_matches_length_times_width() {
    let line = vec![(0.0, 0.0), (2000.0, 0.0)];
    let ring = build_buffered_corridor_polygon(&line, 100.0);
    let expected = 2000.0 * 200.0;
    let area = shoelace_area(&ring);
    assert!(
        (area - expected).abs() / expected < 0.01,
        "area {area} vs {expected}"
    );
}

#[test]
fn swath_width_matches_trigonometry_across_altitudes() {
    for fov in [40.0_f64, 60.0, 90.0] {
        for h in [50.0_f64, 100.0, 250.0, 500.0] {
            let w = compute_camera_swath_width_m(h, Some(fov), None, None);
            let expected = 2.0 * h * (fov.to_radians() / 2.0).tan();
            assert!(
                (w - expected).abs() < 1e-3,
                "h={h} fov={fov}: {w} vs {expected}"
            );
        }
    }
}
