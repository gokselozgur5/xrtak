use xrtak_core::kml_export::{
    export_cot_tracks_kml, export_tracks_to_kml, CotTrack, KmlExportConfig, KmlTrackPoint,
};

#[test]
fn gx_track_has_one_when_per_coord() {
    let mut track = CotTrack::new("UID-1", "HAWK", "a-f-A");
    track.add_point(KmlTrackPoint::new(39.92, 32.85, 100.0).with_time(10_000));
    track.add_point(KmlTrackPoint::new(39.93, 32.86, 120.0)); // no timestamp
    track.add_point(KmlTrackPoint::new(39.94, 32.87, 140.0).with_time(30_000));

    let kml = export_tracks_to_kml(&[track]);
    let whens = kml.matches("<when>").count();
    let coords = kml.matches("<gx:coord>").count();
    assert_eq!(
        whens, coords,
        "gx:Track needs equal numbers of <when> and <gx:coord>"
    );
    assert_eq!(whens, 2);
}

#[test]
fn gx_track_uses_configured_altitude_mode() {
    let mut track = CotTrack::new("UID-1", "HAWK", "a-f-A");
    track.add_point(KmlTrackPoint::new(39.92, 32.85, 100.0).with_time(10_000));

    let cfg = KmlExportConfig {
        altitude_mode: "relativeToGround".to_string(),
        ..Default::default()
    };

    let kml = export_cot_tracks_kml(&[track], &cfg);
    assert!(kml.contains("<altitudeMode>relativeToGround</altitudeMode>"));
}
