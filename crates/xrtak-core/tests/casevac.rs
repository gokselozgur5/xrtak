use xrtak_core::medevac::CotMedevacBuilder;

#[test]
fn test_medevac_cot_xml_serializes_each_line_once() {
    let report = CotMedevacBuilder::new("MED-1", "DUSTOFF")
        .line1_location("GRID123456")
        .build()
        .unwrap();
    let xml = report.to_cot_xml();
    assert_eq!(
        xml.matches("line1=").count(),
        0,
        "line1 should not be serialized as an attribute"
    );
    assert_eq!(
        xml.matches("<line1>").count(),
        1,
        "line1 should be serialized as a child element exactly once"
    );
}
