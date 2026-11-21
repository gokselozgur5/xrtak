use xrtak_core::{DistressBeacon, EmergencySeverity, EmergencyStatus, EmergencyType, SosFlags};

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

#[test]
fn atak_cancel_event_without_type_attribute_parses() {
    let xml = r#"<event version="2.0" uid="ANDROID-1-9-1-1" type="b-a-o-can" how="h-e" time="2026-11-18T10:00:00Z" start="2026-11-18T10:00:00Z" stale="2026-11-18T10:00:10Z"><point lat="34.0" lon="-118.0" hae="100.0" ce="10" le="10"/><detail><emergency cancel="true">EAGLE-6</emergency></detail></event>"#;
    let beacon = xrtak_core::DistressBeacon::from_cot_xml(xml).expect("cancel event parses");
    assert_eq!(beacon.status, xrtak_core::EmergencyStatus::Cancelled);
    assert_eq!(beacon.callsign, "EAGLE-6");
}

#[test]
fn cancel_and_reactivate_keep_stale_after_time() {
    let mut beacon = DistressBeacon::new("B-1", "EAGLE-6", EmergencyType::Alert911, 34.0, -118.0)
        .with_timestamps("2026-11-18T10:00:00Z", "2026-11-18T10:00:10Z");
    beacon.cancel("2026-11-18T10:30:00Z");
    assert!(
        beacon.stale_iso > beacon.time_iso,
        "cancel: time={} stale={}",
        beacon.time_iso,
        beacon.stale_iso
    );
    beacon.activate("2026-11-18T11:00:00Z");
    assert!(
        beacon.stale_iso > beacon.time_iso,
        "activate: stale={}",
        beacon.stale_iso
    );
}

#[test]
fn default_beacon_emits_current_time_not_stale_in_past() {
    let b = DistressBeacon::new("B1", "CS", EmergencyType::Alert911, 34.0, -118.0);
    let xml = b.to_cot_xml();
    assert!(!xml.contains("2026-01-01T00:10:00Z"));
    assert!(b.stale_iso.as_str() > "2026-11-01T00:00:00Z");
}

#[test]
fn severity_and_sos_flags_survive_cot_round_trip() {
    let mut flags = SosFlags::new();
    flags.officer_down = true;
    let beacon = DistressBeacon::new("B-1", "EAGLE-6", EmergencyType::Medical, 34.0, -118.0)
        .with_severity(EmergencySeverity::Low)
        .with_sos_flags(flags);
    let parsed = DistressBeacon::from_cot_xml(&beacon.to_cot_xml()).expect("round trip");
    assert_eq!(parsed.severity, EmergencySeverity::Low);
    assert_eq!(parsed.sos_flags, flags);
}
