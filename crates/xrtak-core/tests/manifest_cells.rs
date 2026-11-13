use xrtak_core::manifest::format_copernicus_glo30_cell_id;

#[test]
fn cell_id_zero_padding_and_hemispheres() {
    let ne = format_copernicus_glo30_cell_id(5, 7);
    assert_eq!(ne, "Copernicus_DSM_COG_10_N05_00_E007_00_DEM");
    let nw = format_copernicus_glo30_cell_id(9, -4);
    assert_eq!(nw, "Copernicus_DSM_COG_10_N09_00_W004_00_DEM");
    let se = format_copernicus_glo30_cell_id(-3, 12);
    assert_eq!(se, "Copernicus_DSM_COG_10_S03_00_E012_00_DEM");
    let sw = format_copernicus_glo30_cell_id(-12, -77);
    assert_eq!(sw, "Copernicus_DSM_COG_10_S12_00_W077_00_DEM");
}
