use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use xrtak_core::grid_convergence::{
    grid_to_true_azimuth, true_to_grid_azimuth, utm_zone_for_lat_lon, utm_zone_from_lon,
};

#[test]
fn test_large_input_angle_normalisation_terminates() {
    let (tx, rx) = channel();
    thread::spawn(move || {
        let large_val = 1e20;
        let _ = utm_zone_from_lon(large_val);
        let _ = utm_zone_for_lat_lon(60.0, large_val);
        let _ = true_to_grid_azimuth(large_val, 0.0);
        let _ = grid_to_true_azimuth(large_val, 0.0);
        tx.send(()).unwrap();
    });

    rx.recv_timeout(Duration::from_millis(500))
        .expect("angle normalisation timed out");
}

fn finishes<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(3)).ok()
}

#[test]
fn azimuth_normalisation_terminates_for_large_and_non_finite_inputs() {
    let az = finishes(|| true_to_grid_azimuth(1.0e19, 0.0)).expect("terminates");
    assert!((0.0..360.0).contains(&az));
    assert_eq!(true_to_grid_azimuth(0.0, 1.0e-15), 0.0);
    assert_eq!(grid_to_true_azimuth(-1.0e-15, 0.0), 0.0);
    let extreme = true_to_grid_azimuth(f64::MAX, -f64::MAX);
    assert!((0.0..360.0).contains(&extreme));
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(true_to_grid_azimuth(bad, 0.0).is_nan());
        assert!(true_to_grid_azimuth(0.0, bad).is_nan());
        assert!(grid_to_true_azimuth(bad, 0.0).is_nan());
        assert!(grid_to_true_azimuth(0.0, bad).is_nan());
    }
}

#[test]
fn zone_lookup_terminates_for_large_and_non_finite_inputs() {
    let z = finishes(|| utm_zone_from_lon(1.0e19)).expect("terminates");
    assert!((1..=60).contains(&z));
    let exact_prime_meridian = 360.0 * ((1_u64 << 50) as f64);
    assert_eq!(utm_zone_from_lon(exact_prime_meridian), 31);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(utm_zone_from_lon(bad), 0);
        assert_eq!(utm_zone_for_lat_lon(60.0, bad), 0);
        assert_eq!(utm_zone_for_lat_lon(bad, 10.0), 0);
    }
}
