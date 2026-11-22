use xrtak_core::geopackage::{GeoPackageFeatureDataset, GeoPackageWaypoint};

#[test]
fn non_finite_elevation_does_not_reach_the_sql_script() {
    let mut ds = GeoPackageFeatureDataset::new("waypoints");
    ds.add_waypoint(GeoPackageWaypoint::new("OP", 36.2, 33.5).with_elevation(f64::NAN));
    match ds.export_sql_script() {
        Err(_) => {}
        Ok(sql) => assert!(!sql.contains("NaN"), "NaN literal in SQL"),
    }
}

#[test]
fn table_name_cannot_inject_statements() {
    let ds = GeoPackageFeatureDataset::new("wp (id INTEGER); DROP TABLE gpkg_contents; --");
    match ds.export_sql_script() {
        Err(_) => {}
        Ok(sql) => assert!(
            !sql.contains("DROP TABLE gpkg_contents; --"),
            "raw table name in SQL"
        ),
    }
}

#[test]
fn geometry_columns_z_flag_matches_waypoint_elevations() {
    let mut ds_2d = GeoPackageFeatureDataset::new("points_2d");
    ds_2d.add_waypoint(GeoPackageWaypoint::new("WP1", 36.2, 33.5));
    let sql_2d = ds_2d.export_sql_script().expect("export 2d script");
    assert!(
        sql_2d.contains("VALUES ('points_2d', 'geom', 'POINT', 4326, 0, 0);"),
        "expected z=0 for 2d dataset"
    );

    let mut ds_3d = GeoPackageFeatureDataset::new("points_3d");
    ds_3d.add_waypoint(GeoPackageWaypoint::new("WP2", 36.2, 33.5).with_elevation(120.0));
    let sql_3d = ds_3d.export_sql_script().expect("export 3d script");
    assert!(
        sql_3d.contains("VALUES ('points_3d', 'geom', 'POINT', 4326, 1, 0);"),
        "expected z=1 for 3d dataset"
    );
}

#[test]
fn export_sql_script_rejects_non_wgs84_srs_id() {
    let mut ds = GeoPackageFeatureDataset::new("waypoints").with_srs_id(3857);
    ds.add_waypoint(GeoPackageWaypoint::new("WP", 34.5, 31.2));
    assert!(
        ds.export_sql_script().is_err(),
        "expected export_sql_script to reject non-WGS84 srs_id"
    );
}
