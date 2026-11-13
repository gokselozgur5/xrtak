use xrtak_core::geo::ElevationProvider;
use xrtak_core::rf::{evaluate_rf_link, RfBand};

struct FlatPlane;
impl ElevationProvider for FlatPlane {
    fn sample_relief_m(&self, _e: f64, _n: f64) -> f64 {
        0.0
    }
}

#[test]
fn flat_terrain_positive_margin_uhf_and_ism() {
    let p = FlatPlane;
    let tx = (0.0, 0.0, 50.0);
    let rx = (500.0, 0.0, 50.0);
    let uhf = evaluate_rf_link(tx, rx, RfBand::UHF900, 20, &p);
    assert!(uhf.link_margin_db > 0.0);
    assert_eq!(uhf.status, "Solid");
    let ism = evaluate_rf_link(tx, rx, RfBand::ISM2400, 20, &p);
    assert!(ism.link_margin_db > 0.0);
    assert_eq!(ism.status, "Solid");
}

#[test]
fn flat_terrain_positive_margin_shf_and_link_metrics() {
    let p = FlatPlane;
    let tx = (0.0, 0.0, 50.0);
    let rx = (500.0, 0.0, 50.0);
    let shf = evaluate_rf_link(tx, rx, RfBand::SHF5800, 20, &p);
    assert!(shf.link_margin_db > 0.0);
    assert_eq!(shf.status, "Solid");
    assert!((shf.distance_m - 500.0).abs() < 1e-3);
    assert_eq!(shf.diffraction_loss_db, 0.0);
    assert!(shf.max_fresnel_radius_m > 0.0);
}
