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
    let r = b
        .nationality(PatientNationality::NonUsMilitary)
        .build()
        .unwrap();
    let d = CotMedevacReport::from_cot_xml(&r.to_cot_xml()).unwrap();
    assert_eq!(
        d.line8_patient_nationality,
        PatientNationality::NonUsMilitary
    );
}

#[test]
fn line7_smoke_colour_survives_round_trip() {
    let b = CotMedevacBuilder::new("M1", "D1");
    let r = b
        .marking(HlzMarking::Smoke("Green".into()))
        .build()
        .unwrap();
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

#[test]
fn medevac_parser_rejects_bad_latitude() {
    const TAIL: &str =
        "lon=\"43.1\" hae=\"0\"/><detail><medevac line1=\"38SMB12345678\"/></detail></event>";
    for lat in ["abc", "95.0", "NaN"] {
        let xml = format!("<event version=\"2.0\" uid=\"M\"><point lat=\"{lat}\" {TAIL}");
        let res = CotMedevacReport::from_cot_xml(&xml);
        assert!(
            res.is_err(),
            "lat={lat} accepted as {:?}",
            res.map(|r| r.lat)
        );
    }
}

#[test]
fn medevac_builder_rejects_nan_latitude() {
    let res = CotMedevacBuilder::new("1", "A")
        .coordinates(f64::NAN, 10.0, 0.0)
        .build();
    assert!(res.is_err());
}

#[test]
fn cas_parser_rejects_unparsable_target_latitude() {
    let xml = "<event uid=\"C\"><point lat=\"abc\" lon=\"-115.0\" hae=\"0\"/>\
        <detail><cas9line line6=\"38SMC99887766\"/></detail></event>";
    let res = xrtak_core::cas_nine_line::CasNineLine::from_cot_xml(xml);
    assert!(res.is_err(), "lat=abc accepted as {:?}", res.map(|b| b.lat));
}

fn attr<'a>(xml: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = xml.find(&key).expect("attribute present") + key.len();
    &xml[start..start + xml[start..].find('"').unwrap()]
}

#[test]
fn medevac_event_is_not_stale_at_start() {
    let xml = CotMedevacBuilder::new("MED-1", "DUSTOFF-1")
        .time_iso("2026-11-16T09:00:00Z")
        .build()
        .unwrap()
        .to_cot_xml();
    assert!(
        attr(&xml, "stale") > attr(&xml, "start"),
        "stale={}",
        attr(&xml, "stale")
    );
}

#[test]
fn cas_event_is_not_stale_at_start() {
    let xml = xrtak_core::cas_nine_line::CasNineLine::new("CAS-1", "WARRIOR 21")
        .with_time("2026-11-16T09:00:00Z")
        .to_cot_xml();
    assert!(
        attr(&xml, "stale") > attr(&xml, "start"),
        "stale={}",
        attr(&xml, "stale")
    );
}
