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

#[test]
fn test_kml_placemark_coordinates() {
    let units = vec![
        sample_unit("GND-1", 45.123456, 12.654321, 55.5),
        sample_unit("GND-2", -10.0, 20.0, 30.0),
    ];
    let kml = generate_kml_tactical_scene("Alpha", &units, (0.0, 0.0, 0.0));
    assert!(kml.contains("<name>GND-1</name>"));
    assert!(kml.contains("<name>GND-2</name>"));
    assert!(kml.contains("<coordinates>12.654321,45.123456,55.5</coordinates>"));
    assert!(kml.contains("<coordinates>20.000000,-10.000000,30.0</coordinates>"));
}

#[test]
fn test_nato_opord_five_numbered_paragraphs() {
    let units = vec![sample_unit("SCOUT-1", 39.0, 32.0, 900.0)];
    let doc = generate_nato_opord_markdown("Anvil", (39.0, 32.0, 900.0), &units, 2, 1);
    assert!(doc.contains("## 1. SITUATION"));
    assert!(doc.contains("## 2. MISSION"));
    assert!(doc.contains("## 3. EXECUTION"));
    assert!(doc.contains("## 4. ADMINISTRATION & LOGISTICS"));
    assert!(doc.contains("## 5. COMMAND & SIGNAL"));
    assert!(doc.contains("1 active tactical units in theatre"));
    assert!(doc.contains("1 air-defense/SAM threat domes active"));
    assert!(doc.contains("2 tactical geofence/ROZ zones established"));
}

#[test]
fn test_opord_bundle_contains_kml_and_opord() {
    let units = vec![sample_unit("HQ-1", 41.0, 29.0, 120.0)];
    let b = build_opord_bundle("Shield", (41.0, 29.0, 120.0), &units, 4, 3);
    assert_eq!(b.unit_count, 1);
    assert_eq!(b.zone_count, 4);
    assert_eq!(b.dome_count, 3);
    assert!(b.kml_scene.contains("<kml"));
    assert!(b.opord_markdown.contains("NATO OPERATION ORDER"));
    assert!(!b.generated_utc.is_empty());
}
