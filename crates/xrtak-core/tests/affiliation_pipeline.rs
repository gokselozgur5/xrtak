use xrtak_core::affiliation::{
    AffiliationMapper, CoTAffiliation, ContactMarker, MarkerShapeGlyph,
    TacticalColor, TacticalThreatLevel,
};

#[test]
fn test_integration_marker_lifecycle() {
    let cot_event = r#"<event uid="RADAR-01" type="a-h-A-M-F"><point lat="34.0" lon="-118.0" hae="5000.0"/><detail><contact callsign="BOGEY-99"/><color argb="-65536"/></detail></event>"#;
    let marker = AffiliationMapper::parse_cot_xml(cot_event).expect("valid xml");
    assert_eq!(marker.uid, "RADAR-01");
    assert_eq!(marker.callsign, "BOGEY-99");
    assert_eq!(marker.affiliation, CoTAffiliation::Hostile);
    assert_eq!(marker.shape, MarkerShapeGlyph::Diamond);
    assert_eq!(marker.threat_level, TacticalThreatLevel::High);
}
