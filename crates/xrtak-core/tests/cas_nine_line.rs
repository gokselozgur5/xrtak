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
