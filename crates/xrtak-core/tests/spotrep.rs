use xrtak_core::{format_salute_report_text, SpotReport};

#[test]
fn test_integration_spotrep_pipeline() {
    let mut r = SpotReport::new("COT-SPOT-99", "RECON-1")
        .with_coordinates(35.5, -114.5, 450.0)
        .with_time("2026-11-17T08:00:00Z")
        .with_assessment("Reconnaissance in force")
        .with_remarks("Monitoring from ridge line");
    r.size = "Company minus (approx 60 PAX)".into();
    r.activity = "Establishing defensive positions".into();
    r.location = "11SMC98765432".into();
    r.unit = "Unknown motorized infantry".into();
    r.time_observed = "170800Z NOV 26".into();
    r.equipment = "3x BTR-80, 2x technicals".into();

    let xml = r.to_cot_xml();
    let parsed = SpotReport::from_cot_xml(&xml).expect("parse failed");
    assert_eq!(parsed.cot_uid, "COT-SPOT-99");
    assert_eq!(parsed.observer_callsign, "RECON-1");
    assert_eq!(parsed.size, "Company minus (approx 60 PAX)");

    let text = format_salute_report_text(&parsed);
    assert!(text.contains("1. S - SIZE: Company minus (approx 60 PAX)"));
}

#[test]
fn test_integration_spotrep_atak_xml_variants() {
    let raw_xml = r#"<event version="2.0" uid="ATAK-SPOT-11" type="b-r-f-h-c" time="2026-11-17T08:30:00Z">
<point lat="36.12" lon="-115.14" hae="620.0"/>
<detail>
  <contact callsign="SENTRY-04"/>
  <remarks>Possible forward observation post</remarks>
  <spotrep size="4 personnel" activity="Static observation" location="11SMC11223344"
           unit="Spetsnaz" time="170830Z NOV 26" equipment="Sniper rifle, binoculars"
           assessment="Spotting for artillery"/>
</detail>
</event>"#;
    let spot = SpotReport::from_cot_xml(raw_xml).expect("parse failed");
    assert_eq!(spot.cot_uid, "ATAK-SPOT-11");
    assert_eq!(spot.observer_callsign, "SENTRY-04");
    assert_eq!(spot.size, "4 personnel");
    assert_eq!(spot.unit, "Spetsnaz");
    assert_eq!(spot.remarks, "Possible forward observation post");
    assert_eq!(spot.lat, 36.12);
}

fn attr<'a>(xml: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = xml.find(&key).expect("attribute present") + key.len();
    &xml[start..start + xml[start..].find('"').expect("closing quote")]
}

#[test]
fn report_and_chat_events_are_not_stale_on_creation() {
    use xrtak_core::geochat::GeoChatPacket;

    let t = "2026-11-18T10:00:00Z";
    let spot = SpotReport::new("S-1", "OBS").with_time(t).to_cot_xml();
    let chat = GeoChatPacket::new("U-1", "LEAD", "HQ", "M-1", "hi", t).to_cot_xml();
    for xml in [spot, chat] {
        assert!(attr(&xml, "stale") > attr(&xml, "start"), "{xml}");
    }
}
