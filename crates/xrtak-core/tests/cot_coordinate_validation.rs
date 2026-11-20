use xrtak_core::{parse_cot_xml, GeoOrigin};

fn event(lat: &str, lon: &str) -> String {
    format!(
        r#"<event version="2.0" uid="U-1" type="a-f-G"><point lat="{lat}" lon="{lon}" hae="900"/></event>"#
    )
}

#[test]
fn parse_cot_xml_rejects_invalid_coordinates() {
    let cases = [
        ("NaN", "32.85"),
        ("39.92", "inf"),
        ("123.0", "32.85"),
        ("39.92", "-500.0"),
    ];
    for (lat, lon) in cases {
        let res = parse_cot_xml(&event(lat, lon), &GeoOrigin::default(), 0);
        assert!(res.is_err(), "lat={lat} lon={lon} accepted");
    }
}
