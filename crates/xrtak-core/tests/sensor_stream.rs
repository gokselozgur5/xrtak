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

#[test]
fn elements_with_explicit_end_tags_are_accepted() {
    let xml = concat!(
        r#"<event version="2.0" uid="V1" type="b-i-v" time="t" start="t" stale="s" how="m-g">"#,
        r#"<point lat="1.0" lon="2.0" hae="0.0" ce="1.0" le="1.0"></point><detail>"#,
        r#"<contact callsign="CAM-1"></contact>"#,
        r#"<__video url="rtsp://h:554/live" uid="u" alias="Cam">"#,
        r#"<ConnectionEntry uid="u" alias="Cam" protocol="rtsp" address="h" port="554" path="/live"/>"#,
        r#"</__video></detail></event>"#
    );
    let desc = parse_sensor_stream_cot_xml(xml).expect("equivalent XML form must parse");
    assert_eq!(desc.lat, 1.0);
    assert_eq!(desc.callsign, "CAM-1");
    assert_eq!(desc.video.url, "rtsp://h:554/live");
}

#[test]
fn default_descriptor_does_not_carry_a_fixed_calendar_date() {
    let conn = xrtak_core::sensor_stream::VideoStreamConnection::new("rtsp://10.0.0.1/live", "Cam");
    let desc = xrtak_core::sensor_stream::SensorStreamDescriptor::new("D1", "C1", 1.0, 2.0, 3.0, conn);
    let xml = desc.to_cot_xml();
    assert!(!desc.time_iso.starts_with("2026-11-16"), "time {}", desc.time_iso);
    assert!(!desc.stale_iso.starts_with("2026-11-16"), "stale {}", desc.stale_iso);
    assert!(!xml.contains("2026-11-16"), "fixed date in {xml}");
}
