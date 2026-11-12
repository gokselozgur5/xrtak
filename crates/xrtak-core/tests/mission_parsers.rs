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
