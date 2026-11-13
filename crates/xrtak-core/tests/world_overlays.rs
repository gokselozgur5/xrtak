use xrtak_core::{Affiliation, Dimension, TacticalEntity};

#[allow(dead_code)]
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
