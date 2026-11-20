use xrtak_core::geo::ProceduralReliefModel;
use xrtak_core::*;

#[test]
fn generated_events_carry_required_time_attributes() {
    let origin = GeoOrigin::default();
    let report = SapientDetectionReport {
        node_id: "N-1".into(),
        object_id: "O-1".into(),
        classification: "TANK".into(),
        confidence: 0.9,
        east_m: 0.0,
        north_m: 0.0,
        alt_offset_m: None,
        speed_mps: None,
        course_deg: None,
        auto_slew_sensor_uid: None,
    };
    let events = [
        build_spi_cot_xml("UAV-1", "FALCON", "a-f-A", 39.92, 32.85, 900.0),
        build_laser_pointer_cot_xml("OP-1", 39.92, 32.85, 900.0),
        build_sapient_cot_xml(&report, &origin, &ProceduralReliefModel),
    ];
    for xml in events {
        for attr in [" time=\"", " start=\"", " stale=\""] {
            assert!(xml.contains(attr), "missing{attr} in {xml}");
        }
    }
}
