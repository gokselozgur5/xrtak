use xrtak_core::geochat::GeoChatPacket;
use xrtak_core::{build_laser_pointer_cot_xml, parse_cot_xml, GeoOrigin};

#[test]
fn laser_pointer_callsign_cannot_override_event_type() {
    let callsign = "A\"\ttype=\"a-h-G-E-S";
    let xml = build_laser_pointer_cot_xml(callsign, 39.92, 32.85, 900.0);
    let ent = parse_cot_xml(&xml, &GeoOrigin::default(), 0).expect("well-formed CoT");
    assert_eq!(ent.cot_type, "b-m-p-c", "xml: {xml}");
}

#[test]
fn geochat_recipient_cannot_add_attributes() {
    let pkt = GeoChatPacket::new("U-1", "LEAD", "HQ", "M-1", "hi", "2026-11-16T12:00:00Z")
        .add_recipient("U-2\" uid2=\"U-EVIL");
    let parsed = GeoChatPacket::from_cot_xml(&pkt.to_cot_xml()).expect("well-formed CoT");
    assert_eq!(parsed.recipients.len(), 1, "{:?}", parsed.recipients);
}
