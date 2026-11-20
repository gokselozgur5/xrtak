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
}

fn child_tags_medevac_sample_xml() -> &'static str {
    "<event version=\"2.0\" uid=\"9LINE-007\" type=\"b-r-f-h-c\" time=\"2026-11-17T08:00:00Z\">\
<point lat=\"32.123456\" lon=\"-110.987654\" hae=\"750.5\" ce=\"9999999\" le=\"9999999\"/>\
<detail><contact callsign=\"MEDIC-7\"/><remarks>Heavy dust</remarks>\
<medevac><line1>GRID-AZ-1234</line1><line2>34.00 MHz</line2>\
<line3>C - PRIORITY</line3><line4>Hoist</line4>\
<urgent>0</urgent><priority>3</priority><routine>0</routine>\
<litter>1</litter><ambulatory>2</ambulatory>\
<line6>E - Enemy troops</line6><line7>Smoke Red</line7>\
<line8>Non-US Military</line8><line9>Rough desert terrain</line9>\
</medevac></detail></event>"
}

#[test]
fn test_integration_medevac_xml_child_tags_parsing() {
    let xml = child_tags_medevac_sample_xml();
    let rep = CotMedevacReport::from_cot_xml(xml).expect("parse child tags");
    assert_eq!(rep.callsign, "MEDIC-7");
    assert_eq!(rep.line1_location, "GRID-AZ-1234");
    assert_eq!(rep.total_patients_by_precedence(), 3);
    assert_eq!(rep.total_patients_by_type(), 3);
    assert!(!rep.is_urgent());
}

#[test]
fn line8_non_us_military_survives_round_trip() {
    let b = CotMedevacBuilder::new("M1", "D1");
    let r = b.nationality(PatientNationality::NonUsMilitary).build().unwrap();
    let d = CotMedevacReport::from_cot_xml(&r.to_cot_xml()).unwrap();
    assert_eq!(d.line8_patient_nationality, PatientNationality::NonUsMilitary);
}

#[test]
fn line7_smoke_colour_survives_round_trip() {
    let b = CotMedevacBuilder::new("M1", "D1");
    let r = b.marking(HlzMarking::Smoke("Green".into())).build().unwrap();
    let d = CotMedevacReport::from_cot_xml(&r.to_cot_xml()).unwrap();
    assert_eq!(d.line7_hlz_marking, HlzMarking::Smoke("Green".into()));
}

#[test]
fn remarks_with_literal_entity_text_survive_round_trip() {
    let b = CotMedevacBuilder::new("M1", "D1");
    let r = b.remarks("type &lt; to page").build().unwrap();
    let d = CotMedevacReport::from_cot_xml(&r.to_cot_xml()).unwrap();
    assert_eq!(d.remarks, "type &lt; to page");
}


