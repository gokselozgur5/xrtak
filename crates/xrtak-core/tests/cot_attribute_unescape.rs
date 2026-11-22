use xrtak_core::geochat::GeoChatPacket;
use xrtak_core::{parse_cot_xml, GeoOrigin};

#[test]
fn parse_cot_xml_unescapes_attribute_values() {
    let xml = r#"<event version="2.0" uid="U-1" type="a-f-G"><point lat="39.92" lon="32.85" hae="900"/><detail><contact callsign="ALPHA &amp; BRAVO"/><__video url="rtsp://10.0.0.1/s?a=1&amp;b=2"/></detail></event>"#;
    let ent = parse_cot_xml(xml, &GeoOrigin::default(), 0).expect("valid CoT");
    assert_eq!(ent.callsign, "ALPHA & BRAVO");
    assert_eq!(ent.video_url.as_deref(), Some("rtsp://10.0.0.1/s?a=1&b=2"));
}

#[test]
fn geochat_sender_callsign_round_trips() {
    let pkt = GeoChatPacket::new(
        "U-1",
        "ALPHA & BRAVO",
        "HQ",
        "M-1",
        "hi",
        "2026-11-16T12:00:00Z",
    );
    let parsed = GeoChatPacket::from_cot_xml(&pkt.to_cot_xml()).expect("round trip");
    assert_eq!(parsed.sender_callsign, "ALPHA & BRAVO");
}
