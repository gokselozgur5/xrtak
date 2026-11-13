use xrtak_core::*;

#[test]
fn test_spi_cot_builder_roundtrip() {
    let origin = GeoOrigin::default();
    let xml = build_spi_cot_xml(
        "ALPHA-1",
        "Alpha 1",
        "a-f-A-M-F-Q",
        39.925012,
        32.855034,
        920.5,
    );
    let entity = parse_cot_xml(&xml, &origin, 1_700_000_000_000).expect("valid SPI CoT XML");
    assert_eq!(entity.uid, "ALPHA-1.SPI1");
    assert_eq!(entity.callsign, "Alpha 1.SPI1");
    assert_eq!(entity.cot_type, "b-m-p-s-p-i");
    assert_eq!(entity.parent_uid.as_deref(), Some("ALPHA-1"));
    assert!((entity.lat - 39.925012).abs() < 1e-6);
    assert!((entity.lon - 32.855034).abs() < 1e-6);
    assert!((entity.hae_m - 920.5).abs() < 0.2);
    assert_eq!(entity.remarks, "SPI Gimbal Lock Target");
}

#[test]
fn test_laser_pointer_cot_builder_roundtrip() {
    let origin = GeoOrigin::default();
    let xml = build_laser_pointer_cot_xml("JTAC Lead", 39.919876, 32.849123, 865.0);
    let entity = parse_cot_xml(&xml, &origin, 1_700_000_000_000).expect("valid laser CoT XML");
    assert_eq!(entity.uid, "XR-LASER-JTAC-Lead");
    assert_eq!(entity.callsign, "LASER-JTAC-Lead");
    assert_eq!(entity.cot_type, "b-m-p-c");
    assert_eq!(entity.parent_uid, None);
    assert!((entity.lat - 39.919876).abs() < 1e-6);
    assert!((entity.lon - 32.849123).abs() < 1e-6);
    assert!((entity.hae_m - 865.0).abs() < 0.2);
    assert_eq!(entity.remarks, "Quest 3 XR Spatial Laser Pointer");
}

#[test]
fn test_cot_builders_coordinate_precision() {
    let origin = GeoOrigin::default();
    let (lat, lon, hae) = (-12.345678, 123.456789, 150.3);
    let spi = build_spi_cot_xml("UAV-9", "Reaper", "a-f-A", lat, lon, hae);
    let ent_spi = parse_cot_xml(&spi, &origin, 0).expect("parse SPI");
    assert!((ent_spi.lat - lat).abs() < 1e-6);
    assert!((ent_spi.lon - lon).abs() < 1e-6);
    assert!((ent_spi.hae_m - hae).abs() < 0.2);

    let laser = build_laser_pointer_cot_xml("Operator 1", lat, lon, hae);
    let ent_laser = parse_cot_xml(&laser, &origin, 0).expect("parse laser");
    assert!((ent_laser.lat - lat).abs() < 1e-6);
    assert!((ent_laser.lon - lon).abs() < 1e-6);
    assert!((ent_laser.hae_m - hae).abs() < 0.2);
}
