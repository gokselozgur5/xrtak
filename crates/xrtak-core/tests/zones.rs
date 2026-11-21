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
