use xrtak_core::{Affiliation, Dimension, GeoOrigin, TacticalEntity, TacticalWorld};

fn make_ent(id: &str, cot: &str, af: Affiliation, e: f64, u: f64, n: f64) -> TacticalEntity {
    TacticalEntity {
        uid: id.into(),
        callsign: id.into(),
        cot_type: cot.into(),
        affiliation: af,
        dimension: Dimension::Ground,
        lat: 0.0,
        lon: 0.0,
        hae_m: u,
        course_deg: 0.0,
        speed_mps: 0.0,
        remarks: String::new(),
        sensor: None,
        video_url: None,
        parent_uid: None,
        east_m: e,
        up_m: u,
        north_m: n,
        updated_unix_ms: 1000,
    }
}

#[test]
fn test_threat_dome_flags_engaged_friendly() {
    let world = TacticalWorld::new(GeoOrigin::default());
    let u0 = world.terrain.sample_relief_m(0.0, 0.0);
    let u1 = world.terrain.sample_relief_m(100.0, 100.0) + 80.0;
    let em = make_ent("RADAR-1", "a-h-G-E-S", Affiliation::Hostile, 0.0, u0, 0.0);
    let tgt = make_ent("UAV-1", "a-f-A", Affiliation::Friendly, 100.0, u1, 100.0);
    let far = make_ent("UAV-2", "a-f-A", Affiliation::Friendly, 900.0, u1, 900.0);
    let (domes, _) = world.evaluate_overlays_for_entities(&[em, tgt, far]);
    assert_eq!(domes.len(), 1);
    assert_eq!(domes[0].engaged_uids, vec!["UAV-1"]);
}

#[test]
fn test_tactical_zone_containment_flagged() {
    let world = TacticalWorld::new(GeoOrigin::default());
    let aff = Affiliation::Friendly;
    let inside = make_ent("TRK-1", "a-f-G", aff, -100.0, 50.0, -100.0);
    let outside = make_ent("TRK-2", "a-f-G", aff, 0.0, 50.0, 500.0);
    let spi = make_ent("SPI-1", "b-m-p-s-p-i", aff, -100.0, 50.0, -100.0);
    let (_, zones) = world.evaluate_overlays_for_entities(&[inside, outside, spi]);
    let roz = zones
        .iter()
        .find(|z| z.uid == "ZONE-ROZ-KILLBOX-1")
        .unwrap();
    assert_eq!(roz.inside_uids, vec!["TRK-1"]);
}

#[test]
fn test_visor_hud_cues_exclude_observer_and_sort_by_uid() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    let e_z = make_ent("ZULU", "a-f-G", Affiliation::Friendly, 10.0, 0.0, 10.0);
    let e_b = make_ent("BRAVO", "a-f-G", Affiliation::Friendly, 20.0, 0.0, 20.0);
    let e_a = make_ent("ALPHA", "a-f-G", Affiliation::Friendly, 30.0, 0.0, 30.0);
    let obs = make_ent("OBS-1", "a-f-G", Affiliation::Friendly, 0.0, 0.0, 0.0);
    let spi = make_ent("SPI-1", "b-m-p-s-p-i", Affiliation::Friendly, 5.0, 0.0, 5.0);
    for e in [e_z, e_b, e_a, obs, spi] {
        world.entities.insert(e.uid.clone(), e);
    }
    let cues = world.compute_visor_hud_cues("OBS-1");
    let uids: Vec<String> = cues.into_iter().map(|c| c.0).collect();
    assert_eq!(uids, vec!["ALPHA", "BRAVO", "ZULU"]);
}
