use xrtak_core::{
    DistressBeacon, EmergencySeverity, EmergencyStatus, EmergencyType, SosFlags,
};

#[test]
fn test_integration_distress_beacon_lifecycle() {
    let mut flags = SosFlags::new();
    flags.officer_down = true;
    flags.medical_critical = true;

    let mut beacon = DistressBeacon::new(
        "DISTRESS-TAC-01",
        "EAGLE-6",
        EmergencyType::InContact,
        34.0522,
        -118.2437,
    )
    .with_severity(EmergencySeverity::Critical)
    .with_sos_flags(flags)
    .with_description("Ambush at checkpoint Charlie");
