use xrtak_core::{GeoOrigin, TacticalWorld};

fn make_cot(uid: &str, cs: &str, lat: f64, lon: f64, hae: f64) -> String {
    format!("<event version=\"2.0\" uid=\"{uid}\" type=\"a-f-G-U-C\"><point lat=\"{lat}\" lon=\"{lon}\" hae=\"{hae}\" ce=\"5\" le=\"5\"/><detail><contact callsign=\"{cs}\"/></detail></event>")
}

#[test]
fn ingest_creates_new_entity_when_uid_absent() {
    let origin = GeoOrigin {
        lat: 39.92,
        lon: 32.85,
        alt_m: 900.0,
        extent_m: 5000.0,
    };
    let mut world = TacticalWorld::new(origin);
    let xml = make_cot("ALPHA-1", "VIPER-1", 39.921, 32.851, 950.0);
    let ent = world.ingest_cot_xml(&xml, 1000).expect("ingest ok");
    assert_eq!(world.entities.len(), 1);
    assert_eq!(ent.uid, "ALPHA-1");
    assert_eq!(ent.callsign, "VIPER-1");
}

#[test]
fn ingest_upserts_existing_entity_by_uid() {
    let origin = GeoOrigin {
        lat: 39.92,
        lon: 32.85,
        alt_m: 900.0,
        extent_m: 5000.0,
    };
    let mut world = TacticalWorld::new(origin);
    let xml1 = make_cot("ALPHA-1", "VIPER-1", 39.921, 32.851, 950.0);
    world.ingest_cot_xml(&xml1, 1000).expect("first ingest");
    let xml2 = make_cot("ALPHA-1", "VIPER-LEAD", 39.925, 32.855, 960.0);
    let ent = world.ingest_cot_xml(&xml2, 2000).expect("second ingest");
    assert_eq!(world.entities.len(), 1);
    assert_eq!(ent.callsign, "VIPER-LEAD");
    assert_eq!(ent.uid, "ALPHA-1");
    let stored = &world.entities["ALPHA-1"];
    assert_eq!(stored.callsign, "VIPER-LEAD");
    assert!((stored.lat - 39.925).abs() < 1e-6);
}
