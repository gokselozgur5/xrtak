use xrtak_core::zones::*;
use xrtak_core::{Affiliation, GeoOrigin};

#[test]
fn parse_cot_polygon_zone_link_points_to_vertices() {
    let origin = GeoOrigin::default();
    let xml = r#"<event uid="POLY-01" type="u-d-f" how="h-e">
        <point lat="39.9200" lon="32.8500" hae="850.0"/>
        <detail>
            <contact callsign="SECTOR-A"/>
            <link point="39.9180,32.8480,850.0"/>
            <link point="39.9180,32.8520,850.0"/>
            <link point="39.9220,32.8520,850.0"/>
            <link point="39.9220,32.8480,850.0"/>
        </detail>
    </event>"#;
    let zone = parse_cot_polygon_zone(xml, &origin).expect("valid polygon zone");
    assert_eq!(zone.uid, "POLY-01");
    assert_eq!(zone.name, "SECTOR-A");
    assert_eq!(zone.affiliation, Affiliation::Friendly);
    assert_eq!(zone.vertices_en.len(), 4);
    assert!(zone.vertices_en[0].0 < 0.0);
    assert!(zone.vertices_en[0].1 < 0.0);
}

#[test]
fn parse_cot_polygon_zone_fewer_than_three_points_rejected() {
    let origin = GeoOrigin::default();
    let one_point = r#"<event uid="POLY-02" type="u-d-f">
        <point lat="39.9200" lon="32.8500" hae="850.0"/>
        <detail><link point="39.9180,32.8480,850.0"/></detail>
    </event>"#;
    assert!(parse_cot_polygon_zone(one_point, &origin).is_none());

    let two_points = r#"<event uid="POLY-03" type="u-d-f">
        <point lat="39.9200" lon="32.8500" hae="850.0"/>
        <detail>
            <link point="39.9180,32.8480,850.0"/>
            <link point="39.9180,32.8520,850.0"/>
        </detail>
    </event>"#;
    assert!(parse_cot_polygon_zone(two_points, &origin).is_none());
}

#[test]
fn parse_cot_polygon_zone_non_udf_type_rejected() {
    let origin = GeoOrigin::default();
    let non_udf = r#"<event uid="UAV-01" type="a-f-A" how="m-g">
        <point lat="39.9200" lon="32.8500" hae="1200.0"/>
        <detail><contact callsign="VIPER-1"/></detail>
    </event>"#;
    assert!(parse_cot_polygon_zone(non_udf, &origin).is_none());

    let empty_event = r#"<event uid="" type="u-d-f"></event>"#;
    assert!(parse_cot_polygon_zone(empty_event, &origin).is_none());

    let no_links = r#"<event uid="POLY-04" type="u-d-f">
        <point lat="39.9200" lon="32.8500" hae="850.0"/>
        <detail><contact callsign="NO-LINKS"/></detail>
    </event>"#;
    assert!(parse_cot_polygon_zone(no_links, &origin).is_none());
}
