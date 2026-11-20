use xrtak_core::comms_channel::CommsChannelPacket;
use xrtak_core::sensor_stream::parse_sensor_stream_cot_xml;

#[test]
fn escaped_attribute_values_are_decoded_by_the_parsers() {
    let xml = concat!(
        r#"<event version="2.0" uid="V1" type="b-i-v" time="t" start="t" stale="s" how="m-g">"#,
        r#"<point lat="1.0" lon="2.0" hae="0.0" ce="1.0" le="1.0"/><detail>"#,
        r#"<contact callsign="R&amp;S-1"/>"#,
        r#"<__video url="rtsp://h/live?user=op&amp;token=abc" uid="u" alias="Cam"/>"#,
        r#"</detail></event>"#
    );
    let desc = parse_sensor_stream_cot_xml(xml).unwrap();
    assert_eq!(desc.video.url, "rtsp://h/live?user=op&token=abc");
    assert_eq!(desc.callsign, "R&S-1");

    let packet = CommsChannelPacket::new("NET&1", "TOC", "t", "s");
    let parsed = CommsChannelPacket::from_cot_xml(&packet.to_cot_xml()).unwrap();
    assert_eq!(parsed.uid, "NET&1");
}
