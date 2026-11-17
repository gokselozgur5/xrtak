use xrtak_core::medevac::*;

#[test]
fn test_integration_medevac_full_pipeline() {
    let report = CotMedevacBuilder::new("MED-EVAC-99", "ANGEL-1")
        .coordinates(36.14, -115.15, 620.0)
        .line1_location("NV-VEGAS-GRID-88")
        .line2_freq_callsign("121.50 MHz / ANGEL-1")
        .precedence(MedevacPrecedence::UrgentSurgical)
        .patient_counts(1, 0, 0, 1, 0)
        .equipment(MedevacEquipment::Ventilator)
        .security(SiteSecurity::NoEnemy)
        .marking(HlzMarking::Panels)
        .nationality(PatientNationality::UsCivilian)
        .terrain_nbc("Clear rooftop helipad")
        .remarks("Critical trauma")
        .build()
        .unwrap();

    let xml = report.to_cot_xml();
    let decoded = CotMedevacReport::from_cot_xml(&xml).expect("decode failed");
    assert_eq!(decoded.cot_uid, "MED-EVAC-99");
    assert_eq!(decoded.callsign, "ANGEL-1");
    assert!(decoded.is_urgent());
    assert_eq!(decoded.total_patients_by_type(), 1);
