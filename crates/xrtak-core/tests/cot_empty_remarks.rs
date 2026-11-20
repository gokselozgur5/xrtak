use xrtak_core::geochat::GeoChatPacket;
use xrtak_core::{parse_cot_xml, GeoOrigin};

#[test]
fn empty_remarks_element_does_not_capture_later_text() {
    let xml = r#"<event version="2.0" uid="U-1" type="a-f-G"><point lat="39.92" lon="32.85" hae="900"/><detail><remarks/><height_unit>4</height_unit></detail></event>"#;
    let ent = parse_cot_xml(xml, &GeoOrigin::default(), 0).expect("valid CoT");
    assert_eq!(ent.remarks, "");
}

#[test]
fn geochat_empty_remarks_does_not_capture_later_text() {
    let xml = r#"<event version="2.0" uid="GeoChat.U-1.HQ.M-1" type="b-t-f"><point lat="0" lon="0" hae="0"/><detail><__chat messageId="M-1" chatroom="HQ" senderCallsign="LEAD"/><link uid="U-1" relation="p-p"/><remarks/><precision>gps</precision></detail></event>"#;
    let pkt = GeoChatPacket::from_cot_xml(xml).expect("valid chat");
    assert_eq!(pkt.message_text, "");
}
