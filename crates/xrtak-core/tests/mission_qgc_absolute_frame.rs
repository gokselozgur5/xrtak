use xrtak_core::mission::parse_qgc_plan_mission;

#[test]
fn mav_frame_global_altitude_is_absolute() {
    // frame 0 = MAV_FRAME_GLOBAL: altitude above mean sea level. No AltitudeMode key.
    let plan = r#"{"fileType":"Plan","mission":{"items":[
        {"command":16,"frame":0,"params":[0,0,0,null,39.92,32.85,900.0],"type":"SimpleItem"}
    ]}}"#;
    let (_, waypoints) = parse_qgc_plan_mission(plan).expect("valid plan");
    assert_eq!(waypoints[0].alt_m, 900.0);
    assert!(!waypoints[0].alt_is_relative, "900 m AMSL was imported as 900 m above ground");
}
