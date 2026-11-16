use xrtak_core::{format_cas_nine_line_text, CasNineLine};

#[test]
fn test_integration_cas_nine_line_pipeline() {
    let mut cas = CasNineLine::new("COT-CAS-77", "DUDE 01")
        .with_coordinates(36.1716, -115.1391, 610.0)
        .with_time("2026-11-16T12:00:00Z")
        .with_remarks("No fire area 2km North.");
    cas.line1_ip_bp = "FORD".into();
    cas.line2_heading = "045 Offset Right".into();
    cas.line3_distance = "8.2nm".into();
    cas.line4_elevation = "2100MSL".into();
    cas.line5_description = "Command post antenna".into();
    cas.line6_location = "11SMD12345678".into();
    cas.line7_mark_type = "Laser PRF 1688".into();
    cas.line8_friendlies = "Friendlies South 800m".into();
    cas.line9_egress = "Egress SW to IP FORD".into();

    let xml = cas.to_cot_xml();
    let parsed = CasNineLine::from_cot_xml(&xml).expect("parse failed");
    assert_eq!(parsed.cot_uid, "COT-CAS-77");
    assert_eq!(parsed.line1_ip_bp, "FORD");
    assert_eq!(parsed.line7_mark_type, "Laser PRF 1688");

    let text = format_cas_nine_line_text(&parsed);
    assert!(text.contains("1. IP/BP: FORD"));
}

#[test]
fn test_integration_cas_atak_xml_variants() {
    let raw_xml = r#"<event version="2.0" uid="ATAK-CAS-99" type="b-m-p-s-p-loc" time="2026-11-16T12:00:00Z">
<point lat="36.5" lon="-115.5" hae="800.0"/>
<detail>
  <contact callsign="ANVIL 22"/>
  <cas9line line1="DODGE" line2="270" line3="5nm" line4="1500" line5="BMP-2" line6="11SMD99881122"
            line7="IR" line8="None" line9="North" remarks="Danger Close 300m"/>
</detail>
</event>"#;
    let cas = CasNineLine::from_cot_xml(raw_xml).expect("parse failed");
    assert_eq!(cas.cot_uid, "ATAK-CAS-99");
    assert_eq!(cas.jtac_callsign, "ANVIL 22");
    assert_eq!(cas.line1_ip_bp, "DODGE");
    assert_eq!(cas.line5_description, "BMP-2");
    assert_eq!(cas.remarks, "Danger Close 300m");
    assert_eq!(cas.lat, 36.5);
}
