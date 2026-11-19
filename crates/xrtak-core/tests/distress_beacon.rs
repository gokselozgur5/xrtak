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
    assert_eq!(beacon.cot_type(), "b-a-o-opn");
    let xml = beacon.to_cot_xml();
    assert!(xml.contains("b-a-o-opn"));
    assert!(xml.contains("In Contact"));

    let parsed = DistressBeacon::from_cot_xml(&xml).expect("parsing CoT XML failed");
    assert_eq!(parsed.beacon_uid, "DISTRESS-TAC-01");
    assert_eq!(parsed.callsign, "EAGLE-6");
    assert_eq!(parsed.alert_type, EmergencyType::InContact);
    assert_eq!(parsed.status, EmergencyStatus::Active);

    beacon.cancel("2026-11-17T08:00:00Z");
    assert_eq!(beacon.cot_type(), "b-a-o-can");
    let cancel_xml = beacon.to_cot_xml();
    let parsed_cancelled = DistressBeacon::from_cot_xml(&cancel_xml).expect("parse cancel");
    assert_eq!(parsed_cancelled.status, EmergencyStatus::Cancelled);
}
