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
