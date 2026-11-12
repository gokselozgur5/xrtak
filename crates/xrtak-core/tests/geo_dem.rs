use xrtak_core::geo::*;
use xrtak_core::GeoOrigin;

#[test]
fn new_rejects_invalid_grids() {
    assert!(RasterDemProvider::new("a", 1, 2, 10.0, vec![0.0; 2]).is_err());
    assert!(RasterDemProvider::new("a", 2, 2, 0.0, vec![0.0; 4]).is_err());
    assert!(RasterDemProvider::new("a", 2, 2, 10.0, vec![0.0; 3]).is_err());
}

#[test]
fn preset_normalizes_name_and_minimum_resolution() {
    let dem = RasterDemProvider::from_preset(" Alpine ", 100.0, 4);
    assert_eq!(dem.name, "alpine");
    assert_eq!((dem.width, dem.heights_m.len()), (16, 256));
}
