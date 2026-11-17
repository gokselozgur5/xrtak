use xrtak_core::fresnel_zone::*;

#[test]
fn test_fresnel_zone_ellipsoid_clearance() {
    let t = RfTransceiver::default();
    let cfg = FresnelCalculatorConfig::default();
    let obs = vec![ObstaclePoint {
        distance_m: 500.0,
        terrain_elevation_m: 10.0,
        structure_height_m: 5.0,
        name: "Forest_Canopy".to_string(),
    }];

    let res = calculate_rf_link_budget(&t, 50.0, 50.0, 1000.0, &obs, &cfg);
    assert_eq!(res.quality, LinkQuality::Marginal);
    assert_eq!(res.diffraction_loss_db, 0.0);
    assert!(res.worst_clearance_ratio > 1.0);
    let worst = res.worst_point.unwrap();
    assert_eq!(worst.status, ClearanceStatus::Clear);
}

#[test]
fn test_knife_edge_diffraction_severed_link() {
    let t = RfTransceiver {
        tx_power_dbm: 10.0,
        rx_sensitivity_dbm: -90.0,
        fade_margin_db: 15.0,
        ..Default::default()
    };
    let cfg = FresnelCalculatorConfig::default();
    let obs = vec![ObstaclePoint {
        distance_m: 500.0,
        terrain_elevation_m: 120.0,
        structure_height_m: 0.0,
        name: "Mountain_Peak".to_string(),
    }];

    let res = calculate_rf_link_budget(&t, 50.0, 50.0, 1000.0, &obs, &cfg);
    assert_eq!(res.quality, LinkQuality::Severed);
    assert!(res.diffraction_loss_db > 15.0);
    assert!(res.link_margin_db < 0.0);
    let worst = res.worst_point.unwrap();
    assert_eq!(worst.status, ClearanceStatus::Obstructed);
}
