use xrtak_core::manifest::{
    build_mapset_manifest, copernicus_glo30_cells_for_bounds, estimate_tiles_for_bounds,
    ManifestInput,
};
use xrtak_core::tiles::ImageryProvider;

#[test]
fn box_crossing_the_antimeridian_selects_only_the_cells_it_covers() {
    // Taveuni, Fiji: 17 S to 16 S, from 179.5 E eastward across 180 to 179.5 W.
    let cells = copernicus_glo30_cells_for_bounds(-17.0, 179.5, -16.0, -179.5);
    let lons: Vec<i32> = cells.iter().map(|c| c.lon_floor_deg).collect();
    assert_eq!(cells.len(), 2, "selected {} cells", cells.len());
    assert!(lons.contains(&179) && lons.contains(&-180), "{lons:?}");
}

#[test]
fn estimate_tiles_and_manifest_crossing_antimeridian() {
    let est = estimate_tiles_for_bounds(-17.0, 179.5, -16.0, -179.5, 10);
    assert_eq!(est.tiles_x, 4);
    assert_eq!(est.total_tiles, est.tiles_x as u64 * est.tiles_y as u64);

    let m = build_mapset_manifest(&ManifestInput {
        name: "FIJI-TAVEUNI",
        provider: ImageryProvider::BingAerial,
        south_deg: -17.0,
        west_deg: 179.5,
        north_deg: -16.0,
        east_deg: -179.5,
        min_zoom: 10,
        max_zoom: 10,
    });
    assert_eq!(m.bounds_wsen, [179.5, -17.0, -179.5, -16.0]);
    assert_eq!(m.copernicus_dem_cells.len(), 2);
    assert_eq!(m.total_tiles, 4 * est.tiles_y as u64);
}
