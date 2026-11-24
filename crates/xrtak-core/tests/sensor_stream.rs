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
    let desc =
        xrtak_core::sensor_stream::SensorStreamDescriptor::new("D1", "C1", 1.0, 2.0, 3.0, conn);
    let xml = desc.to_cot_xml();
    assert!(
        !desc.time_iso.starts_with("2026-11-16"),
        "time {}",
        desc.time_iso
    );
    assert!(
        !desc.stale_iso.starts_with("2026-11-16"),
        "stale {}",
        desc.stale_iso
    );
    assert!(!xml.contains("2026-11-16"), "fixed date in {xml}");
}

#[test]
fn link_tag_roundtrip_with_custom_parent_cot_type_and_default_fallback() {
    let conn =
        xrtak_core::sensor_stream::VideoStreamConnection::new("rtsp://10.0.0.1/live", "UAS-Cam");
    let desc = xrtak_core::sensor_stream::SensorStreamDescriptor::new(
        "V-1", "CAM-1", 34.0, -118.0, 100.0, conn,
    )
    .with_parent_uid("UAS-ALPHA")
    .with_parent_cot_type("a-u-A-M-F-Q");
    let xml = desc.to_cot_xml();
    assert!(xml.contains(r#"<link relation="p-p" uid="UAS-ALPHA" type="a-u-A-M-F-Q"/>"#));
    let parsed = parse_sensor_stream_cot_xml(&xml).expect("must parse custom parent cot type");
    assert_eq!(parsed.parent_uid.as_deref(), Some("UAS-ALPHA"));
    assert_eq!(parsed.parent_cot_type.as_deref(), Some("a-u-A-M-F-Q"));

    let conn2 =
        xrtak_core::sensor_stream::VideoStreamConnection::new("rtsp://10.0.0.1/live", "UAS-Cam2");
    let desc_default = xrtak_core::sensor_stream::SensorStreamDescriptor::new(
        "V-2", "CAM-2", 34.0, -118.0, 100.0, conn2,
    )
    .with_parent_uid("GND-VEHICLE");
    let xml2 = desc_default.to_cot_xml();
    assert!(xml2.contains(r#"<link relation="p-p" uid="GND-VEHICLE" type="a-f-G-U-C"/>"#));
    let parsed2 = parse_sensor_stream_cot_xml(&xml2).expect("must parse default parent cot type");
    assert_eq!(parsed2.parent_uid.as_deref(), Some("GND-VEHICLE"));
    assert_eq!(parsed2.parent_cot_type.as_deref(), Some("a-f-G-U-C"));
}

#[test]
fn link_tag_escaping_roundtrip() {
    let conn = xrtak_core::sensor_stream::VideoStreamConnection::new("rtsp://10.0.0.1/live", "Cam");
    let desc = xrtak_core::sensor_stream::SensorStreamDescriptor::new(
        "V-3", "CAM-3", 34.0, -118.0, 100.0, conn,
    )
    .with_parent_uid("UAS&HQ<1>")
    .with_parent_cot_type("a-u&special<test>");
    let xml = desc.to_cot_xml();
    assert!(xml.contains(r#"uid="UAS&amp;HQ&lt;1&gt;""#));
    assert!(xml.contains(r#"type="a-u&amp;special&lt;test&gt;""#));
    let parsed = parse_sensor_stream_cot_xml(&xml).expect("must parse escaped link tag");
    assert_eq!(parsed.parent_uid.as_deref(), Some("UAS&HQ<1>"));
    assert_eq!(parsed.parent_cot_type.as_deref(), Some("a-u&special<test>"));
}
