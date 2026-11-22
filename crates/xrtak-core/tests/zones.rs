use xrtak_core::domain::Affiliation;
use xrtak_core::zones::parse_cot_polygon_zone;
use xrtak_core::GeoOrigin;

#[test]
fn parse_cot_polygon_zone_rejects_bmr_route() {
    let origin = GeoOrigin::default();
    let route_xml = r#"<event uid="ROUTE-01" type="b-m-r">
        <point lat="39.9200" lon="32.8500" hae="850.0"/>
        <detail>
            <link point="39.9180,32.8480,850.0"/>
            <link point="39.9180,32.8520,850.0"/>
            <link point="39.9220,32.8520,850.0"/>
        </detail>
    </event>"#;
    assert!(parse_cot_polygon_zone(route_xml, &origin).is_none());
}

#[test]
fn zone_named_secured_lz_is_not_hostile() {
    let xml = r#"<event version="2.0" uid="POLY-1" type="u-d-f" how="h-e">
        <point lat="39.92" lon="32.85" hae="850" ce="10" le="10"/><detail>
        <contact callsign="SECURED LZ"/>
        <link point="39.918,32.848,850"/><link point="39.918,32.852,850"/>
        <link point="39.922,32.852,850"/><link point="39.922,32.848,850"/>
        </detail></event>"#;
    let zone = parse_cot_polygon_zone(xml, &GeoOrigin::default()).expect("valid zone");
    assert_eq!(zone.affiliation, Affiliation::Friendly);
}
