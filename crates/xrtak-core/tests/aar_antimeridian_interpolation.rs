use std::collections::HashMap;
use xrtak_core::aar::AarTimeline;

#[test]
fn interpolation_across_the_antimeridian_stays_near_180_degrees() {
    let origin = xrtak_core::GeoOrigin::default();
    let xml = r#"<event uid="U1" type="a-f-S"><point lat="0.0" lon="179.9" hae="0.0"/></event>"#;
    let mut entity = xrtak_core::parse_cot_xml(xml, &origin, 1000).unwrap();
    let mut timeline = AarTimeline::new(8, 0.1);

    entity.lon = 179.9;
    timeline.maybe_record(
        10.0,
        10_000,
        &HashMap::from([("U1".to_string(), entity.clone())]),
    );
    entity.lon = -179.9;
    timeline.maybe_record(20.0, 20_000, &HashMap::from([("U1".to_string(), entity)]));

    let lon = timeline.sample_at(15.0).unwrap().entities[0].lon;
    assert!(lon.abs() > 179.8, "interpolated longitude {lon}");
}
