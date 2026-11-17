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

#[test]
fn test_integration_multi_contact_batch() {
    let uids = vec!["BLUE-1", "BLUE-2", "RED-1", "NEUT-1", "UNK-1"];
    let types = vec!["a-f-G", "a-a-A", "a-h-G", "a-n-G", "a-u-G"];
    let markers: Vec<ContactMarker> = uids
        .into_iter()
        .zip(types)
        .map(|(uid, ty)| AffiliationMapper::create_marker(uid, uid, ty, None))
        .collect();

    let stats = AffiliationMapper::evaluate_contacts(&markers);
    assert_eq!(stats.total, 5);
    assert_eq!(stats.friendly_count, 2);
    assert_eq!(stats.hostile_count, 1);
    assert_eq!(stats.neutral_count, 1);
    assert_eq!(stats.unknown_count, 1);
}
