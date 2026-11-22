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
