use xrtak_core::{DistressBeacon, EmergencyType};

fn cot_type(kind: EmergencyType) -> &'static str {
    DistressBeacon::new("ANDROID-1-9-1-1", "EAGLE-6", kind, 34.0, -118.0).cot_type()
}

#[test]
fn emergency_types_use_atak_cot_types() {
    assert_eq!(cot_type(EmergencyType::Alert911), "b-a-o-tbl");
    assert_eq!(cot_type(EmergencyType::RingTheBell), "b-a-o-pan");
    assert_eq!(cot_type(EmergencyType::InContact), "b-a-o-opn");
}
