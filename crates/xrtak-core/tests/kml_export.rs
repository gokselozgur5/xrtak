use xrtak_core::geo::ProceduralReliefModel;
use xrtak_core::kml_export::{export_tracks_to_kml, CotTrack, KmlTrackPoint};
use xrtak_core::mission::export_atak_route_cot_xml;
use xrtak_core::GeoOrigin;

#[test]
fn kml_track_names_are_escaped() {
    let mut track = CotTrack::new("UID-1", "R&D <1>", "a-f-G");
    track.add_point(KmlTrackPoint::new(39.92, 32.85, 850.0));
    let kml = export_tracks_to_kml(&[track]);
    assert!(kml.contains("<name>R&amp;D &lt;1&gt; Track</name>"), "raw name in KML");
}

#[test]
fn route_cot_attributes_are_escaped() {
    let xml = export_atak_route_cot_xml(
        "RTE-1",
        "X\" type=\"a-h-G",
        &GeoOrigin::default(),
        &ProceduralReliefModel,
        &[(0.0, 0.0, 100.0)],
    );
    assert!(!xml.contains("callsign=\"X\" type=\"a-h-G\""), "attribute injected: {xml}");
}
