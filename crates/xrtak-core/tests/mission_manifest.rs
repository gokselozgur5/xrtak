use xrtak_core::geochat::GeoChatPacket;

#[test]
fn test_geochat_unescape_numeric_character_references() {
    let pkt = GeoChatPacket::new(
        "U-1",
        "ALPHA",
        "ALL",
        "M-1",
        "hello",
        "2026-11-22T00:00:00Z",
    );
    let xml = pkt
        .to_cot_xml()
        .replace("ALPHA", "LEAD &#38; CO")
        .replace("ALL", "ROOM &#60;ALPHA&#62;")
        .replace("hello", "Roger &#x27;Bravo&#x27;");
    let parsed = GeoChatPacket::from_cot_xml(&xml).expect("parse geochat xml");
    assert_eq!(parsed.sender_callsign, "LEAD & CO");
    assert_eq!(parsed.chatroom, "ROOM <ALPHA>");
    assert_eq!(parsed.message_text, "Roger 'Bravo'");
}
