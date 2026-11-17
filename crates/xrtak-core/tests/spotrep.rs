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
