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

#[test]
fn srtm_normalizes_to_minimum_and_zeroes_voids() {
    let raw: [i16; 4] = [100, 120, -32768, 110];
    let bytes: Vec<u8> = raw.iter().flat_map(|v| v.to_be_bytes()).collect();
    let dem = RasterDemProvider::from_srtm_i16_be("s", 2, 2, 50.0, &bytes).unwrap();
    assert_eq!(dem.heights_m, vec![0.0, 20.0, 0.0, 10.0]);
    assert!(RasterDemProvider::from_srtm_i16_be("s", 2, 2, 50.0, &bytes[..6]).is_err());
}
