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

fn ramp() -> RasterDemProvider {
    RasterDemProvider::new("r", 2, 2, 100.0, vec![0.0, 0.0, 100.0, 100.0]).unwrap()
}

#[test]
fn slope_is_zero_on_flat_and_atan_half_on_ramp() {
    let flat = RasterDemProvider::new("f", 2, 2, 100.0, vec![5.0; 4]).unwrap();
    assert!(flat.sample_slope_deg(0.0, 0.0).abs() < 1e-9);
    assert!((ramp().sample_slope_deg(0.0, 0.0) - 0.5f64.atan().to_degrees()).abs() < 1e-6);
}

#[test]
fn export_grid_has_resolution_squared_samples() {
    let grid = ramp().export_grid(100.0, 3);
    assert_eq!(grid.len(), 9);
    assert_eq!((grid[0], grid[8]), (0.0, 100.0));
}

#[test]
fn profile_handles_empty_and_straight_paths() {
    let empty = compute_elevation_profile(&ramp(), &[], 500.0, 8);
    assert_eq!((empty.min_asl_m, empty.samples.len()), (500.0, 0));
    let p = compute_elevation_profile(&ramp(), &[(0.0, -40.0), (0.0, 40.0)], 0.0, 8);
    assert!((p.total_distance_m - 80.0).abs() < 1e-6);
    assert!(p.total_ascent_m > 0.0 && p.total_descent_m == 0.0);
}

#[test]
fn origin_maps_to_zero_and_round_trips() {
    let o = GeoOrigin::default();
    assert_eq!(o.lla_to_enu(o.lat, o.lon, o.alt_m), (0.0, 0.0, 0.0));
    let (e, u, n) = o.lla_to_enu(o.lat + 0.001, o.lon + 0.001, o.alt_m + 25.0);
    assert!((n - 111.1).abs() < 0.5 && u == 25.0 && e > 80.0);
    let (lat, lon, hae) = o.enu_to_lla(e, u, n);
    assert!((lat - (o.lat + 0.001)).abs() < 1e-9);
    assert!((lon - (o.lon + 0.001)).abs() < 1e-9 && hae == o.alt_m + 25.0);
}
