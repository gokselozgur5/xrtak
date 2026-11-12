use xrtak_core::geo::RasterDemProvider;
use xrtak_core::mission::*;
use xrtak_core::GeoOrigin;

const KML: &str = "<kml><Document><name>Route</name><Placemark><name>Alpha</name>\
<Point><coordinates>32.5,39.9,50</coordinates></Point></Placemark>\
<Placemark><LineString><altitudeMode>relativeToGround</altitudeMode>\
<coordinates>32.1,39.1 32.2,39.2,70</coordinates></LineString></Placemark></Document></kml>";

#[test]
fn kml_parses_names_altitudes_and_modes() {
    let (name, wps) = parse_kml_mission(KML).unwrap();
    assert_eq!(name, "Route");
    assert_eq!(wps.len(), 3);
    assert_eq!(wps[0].name, "Alpha");
    assert!((wps[0].lon - 32.5).abs() < 1e-9 && wps[0].alt_m == 50.0);
    assert!(!wps[0].alt_is_relative && wps[1].alt_is_relative);
    assert_eq!((wps[1].alt_m, wps[2].alt_m), (120.0, 70.0));
}

#[test]
fn kml_without_coordinates_is_rejected() {
    assert!(parse_kml_mission("<kml><Document/></kml>").is_err());
    assert!(parse_kml_mission("<a><b></a>").is_err());
}

#[test]
fn qgc_plan_parses_and_skips_empty_coordinates() {
    let json = r#"{"mission":{"items":[
        {"params":[0,0,0,0,39.9,32.8,100],"frame":3},
        {"params":[0,0,0,0,0,0,50]},
        {"params":[0,0,0,0,40.0,33.0,200],"frame":0,"AltitudeMode":0}]}}"#;
    let (name, wps) = parse_qgc_plan_mission(json).unwrap();
    assert_eq!(name, "QGC-PLAN-MISSION");
    assert_eq!(wps.len(), 2);
    assert_eq!(wps[0].name, "QGC-WP-1");
    assert!(wps[0].alt_is_relative && !wps[1].alt_is_relative);
}

#[test]
fn qgc_plan_rejects_bad_input() {
    assert!(parse_qgc_plan_mission("{").is_err());
    let err = parse_qgc_plan_mission("{}").unwrap_err();
    assert!(err.contains("mission.items"));
}

fn flat() -> RasterDemProvider {
    RasterDemProvider::new("flat", 2, 2, 100.0, vec![0.0; 4]).unwrap()
}

#[test]
fn qgc_export_round_trips_through_parser() {
    let wps = [(0.0, 0.0, 80.0), (50.0, 50.0, 90.0)];
    let json = export_qgc_plan_json(&GeoOrigin::default(), &flat(), &wps);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["mission"]["items"][0]["command"], 22);
    assert_eq!(v["mission"]["items"][1]["command"], 16);
    let (_, back) = parse_qgc_plan_mission(&json).unwrap();
    assert_eq!(back.len(), 2);
}

#[test]
fn route_cot_lists_one_link_per_waypoint() {
    let wps = [(0.0, 0.0, 80.0), (50.0, 50.0, 90.0)];
    let xml = export_atak_route_cot_xml("R1", "Route", &GeoOrigin::default(), &flat(), &wps);
    assert!(xml.contains(r#"type="b-m-r""#));
    assert_eq!(xml.matches("b-m-p-w").count(), 2);
    assert!(!xml.contains("< strokeWeight"));
}
