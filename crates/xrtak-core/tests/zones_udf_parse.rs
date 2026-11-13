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
