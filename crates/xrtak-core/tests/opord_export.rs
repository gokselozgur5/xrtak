use quick_xml::events::Event;
use quick_xml::Reader;
use xrtak_core::opord::*;

fn sample_unit(callsign: &str, lat: f64, lon: f64, alt: f64) -> OpordUnitSummary {
    OpordUnitSummary {
        callsign: callsign.into(),
        cot_type: "a-f-G-U-C".into(),
        affiliation: "friendly".into(),
        lat,
        lon,
        alt_m: alt,
    }
}

#[test]
fn test_kml_scene_parses_with_quick_xml() {
    let units = vec![sample_unit("ALPHA-1", 37.77, -122.41, 150.0)];
    let kml = generate_kml_tactical_scene("Citadel", &units, (37.77, -122.41, 100.0));
    let mut reader = Reader::from_str(&kml);
    let mut count = 0;
    while let Ok(event) = reader.read_event() {
        if matches!(event, Event::Eof) {
            break;
        }
        count += 1;
    }
    assert!(count > 0);
}
