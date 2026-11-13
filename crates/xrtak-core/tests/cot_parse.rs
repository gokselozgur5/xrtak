use xrtak_core::*;

#[test]
fn parse_cot_xml_happy_path_enu_and_fields() {
    let origin = GeoOrigin::default();
    let xml = r#"<event uid="ALPHA-1" type="a-f-G-U-C" how="m-g">
        <point lat="39.9200" lon="32.8500" hae="950.0" ce="10.0" le="5.0"/>
        <detail>
            <contact callsign="VIPER-1"/>
            <track course="90.0" speed="12.5"/>
            <remarks>patrol element</remarks>
        </detail>
    </event>"#;
    let ent = parse_cot_xml(xml, &origin, 5000).expect("parse valid cot");
    assert_eq!(ent.uid, "ALPHA-1");
    assert_eq!(ent.callsign, "VIPER-1");
    assert_eq!(ent.affiliation, Affiliation::Friendly);
    assert_eq!(ent.dimension, Dimension::Ground);
    assert_eq!(ent.east_m, 0.0);
    assert_eq!(ent.north_m, 0.0);
    assert_eq!(ent.up_m, 100.0);
    assert_eq!(ent.course_deg, 90.0);
    assert_eq!(ent.speed_mps, 12.5);
    assert_eq!(ent.remarks, "patrol element");
    assert_eq!(ent.updated_unix_ms, 5000);
}

#[test]
fn parse_cot_xml_missing_point_rejected() {
    let origin = GeoOrigin::default();
    let no_point = r#"<event uid="TGT-01" type="a-h-G">
        <detail><contact callsign="HOSTILE"/></detail>
    </event>"#;
    assert!(parse_cot_xml(no_point, &origin, 1000).is_err());

    let no_lat = r#"<event uid="TGT-02" type="a-h-G">
        <point lon="32.8500" hae="850.0"/>
    </event>"#;
    assert!(parse_cot_xml(no_lat, &origin, 1000).is_err());

    let no_uid = r#"<event type="a-h-G">
        <point lat="39.9200" lon="32.8500"/>
    </event>"#;
    assert!(parse_cot_xml(no_uid, &origin, 1000).is_err());
}

#[test]
fn parse_cot_xml_sensor_cone_attributes() {
    let origin = GeoOrigin::default();
    let xml = r#"<event uid="RECON-1" type="a-f-A">
        <point lat="39.9200" lon="32.8500" hae="1200.0"/>
        <detail>
            <sensor azimuth="135.5" elevation="-25.0" fov="45.0" vfov="30.0" range="1200.0"/>
        </detail>
    </event>"#;
    let ent = parse_cot_xml(xml, &origin, 2000).expect("valid sensor cot");
    let sensor = ent.sensor.expect("sensor cone present");
    assert_eq!(sensor.azimuth_deg, 135.5);
    assert_eq!(sensor.elevation_deg, -25.0);
    assert_eq!(sensor.hfov_deg, 45.0);
    assert_eq!(sensor.vfov_deg, 30.0);
    assert_eq!(sensor.range_m, 1200.0);
}

#[test]
fn parse_cot_xml_video_url_and_parent_link() {
    let origin = GeoOrigin::default();
    let xml = r#"<event uid="SPI-99" type="b-m-p-s-p-i">
        <point lat="39.9200" lon="32.8500" hae="900.0"/>
        <detail>
            <__video url="rtsp://192.168.1.100:8554/live"/>
            <link relation="p-p" uid="PARENT-UAV-1"/>
        </detail>
    </event>"#;
    let ent = parse_cot_xml(xml, &origin, 3000).expect("valid spi cot");
    assert_eq!(
        ent.video_url.as_deref(),
        Some("rtsp://192.168.1.100:8554/live")
    );
    assert_eq!(ent.parent_uid.as_deref(), Some("PARENT-UAV-1"));
}
