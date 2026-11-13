use xrtak_core::sapient::sapient_class_to_cot_type;

#[test]
fn every_known_sapient_class_maps_to_cot_type() {
    let air = ["UAV_ROTARY", "UAV", "DRONE", "UAS"];
    for cls in air {
        assert_eq!(sapient_class_to_cot_type(cls), "a-h-A-M-F-Q");
    }

    let armor = ["HOSTILE_ARMOR", "ARMOR", "TANK", "VEHICLE"];
    for cls in armor {
        assert_eq!(sapient_class_to_cot_type(cls), "a-h-G-E-V-A");
    }

    let emitter = ["RF_JAMMER", "EMITTER", "RADAR", "SAM"];
    for cls in emitter {
        assert_eq!(sapient_class_to_cot_type(cls), "a-h-G-E-S");
    }

    let dismount = ["DISMOUNT_HOSTILE", "INFANTRY", "HUMAN"];
    for cls in dismount {
        assert_eq!(sapient_class_to_cot_type(cls), "a-h-G-U-C-I");
    }
}

#[test]
fn unknown_sapient_classes_fall_back_to_generic() {
    assert_eq!(sapient_class_to_cot_type("UNKNOWN"), "a-u-G");
    assert_eq!(sapient_class_to_cot_type("CIVILIAN_CAR"), "a-u-G");
    assert_eq!(sapient_class_to_cot_type(""), "a-u-G");
    assert_eq!(sapient_class_to_cot_type("???"), "a-u-G");

    assert_eq!(sapient_class_to_cot_type("  drone  "), "a-h-A-M-F-Q");
    assert_eq!(sapient_class_to_cot_type("tank\n"), "a-h-G-E-V-A");
    assert_eq!(sapient_class_to_cot_type("radar"), "a-h-G-E-S");
    assert_eq!(sapient_class_to_cot_type(" human "), "a-h-G-U-C-I");
}

#[test]
fn generated_sapient_cot_xml_parses_cleanly() {
    use xrtak_core::geo::ProceduralReliefModel;
    use xrtak_core::sapient::{build_sapient_cot_xml, SapientDetectionReport};
    use xrtak_core::{parse_cot_xml, Affiliation, GeoOrigin};

    let report = SapientDetectionReport {
        node_id: "RADAR-1".into(),
        object_id: "T90-A".into(),
        classification: "TANK".into(),
        confidence: 0.95,
        east_m: 100.0,
        north_m: 200.0,
        alt_offset_m: Some(0.0),
        speed_mps: Some(12.0),
        course_deg: Some(180.0),
        auto_slew_sensor_uid: None,
    };
    let xml = build_sapient_cot_xml(&report, &GeoOrigin::default(), &ProceduralReliefModel);
    let entity = parse_cot_xml(&xml, &GeoOrigin::default(), 1000).expect("parses");
    assert_eq!(entity.uid, "SAPIENT-T90-A");
    assert_eq!(entity.cot_type, "a-h-G-E-V-A");
    assert_eq!(entity.affiliation, Affiliation::Hostile);
}
