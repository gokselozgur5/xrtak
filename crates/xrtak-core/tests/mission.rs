use xrtak_core::geo::ProceduralReliefModel;
use xrtak_core::mission::export_atak_route_cot_xml;
use xrtak_core::GeoOrigin;

#[test]
fn route_event_carries_required_time_attributes() {
    let xml = export_atak_route_cot_xml(
        "RTE-1",
        "ALPHA",
        &GeoOrigin::default(),
        &ProceduralReliefModel,
        &[(0.0, 0.0, 100.0), (200.0, 100.0, 100.0)],
    );
    let event_tag = &xml[..xml.find('>').expect("event start tag")];
    for attr in [" time=\"", " start=\"", " stale=\""] {
        assert!(event_tag.contains(attr), "missing{attr} in {event_tag}");
    }
}
