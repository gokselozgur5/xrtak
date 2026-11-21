use xrtak_core::corridor::evaluate_swath_corridor;
use xrtak_core::opord::{generate_kml_tactical_scene, OpordUnitSummary};
use xrtak_core::GeoOrigin;

#[test]
fn kml_scene_escapes_callsign() {
    let unit = OpordUnitSummary {
        callsign: "X</name><NetworkLink><name>Y".into(),
        cot_type: "a-f-G".into(),
        affiliation: "Friendly".into(),
        lat: 39.9,
        lon: 32.8,
        alt_m: 900.0,
    };
    let kml = generate_kml_tactical_scene("R&D", &[unit], (39.9, 32.8, 900.0));
    assert!(
        !kml.contains("<NetworkLink>"),
        "callsign injected an element"
    );
    assert!(
        kml.contains("Scene: R&amp;D</name>"),
        "scene name not escaped"
    );
}

#[test]
fn corridor_kml_escapes_name() {
    let line = [(0.0, 0.0), (1000.0, 0.0)];
    let origin = GeoOrigin::default();
    let res = evaluate_swath_corridor("R&D", &origin, &line, 5.0, 100.0, Some(90.0), None, None);
    assert!(
        res.kml_polygon.contains("<name>R&amp;D</name>"),
        "corridor name not escaped"
    );
}
