use crate::domain::{Affiliation, TacticalZone, ThreatDome};
use crate::geo::ElevationProvider;
use crate::viewshed::compute_link_los;
use crate::GeoOrigin;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Parse an ATAK polygonal drawing CoT (`u-d-f` / `u-d-r` with `<link point="lat,lon,hae"/>`) into a 3D `TacticalZone` (`REQ-TAC-03`).
pub fn parse_cot_polygon_zone(xml: &str, origin: &GeoOrigin) -> Option<TacticalZone> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut uid = String::new();
    let mut cot_type = String::new();
    let mut callsign = String::new();
    let mut vertices_en = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"event" => {
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"uid" => uid = String::from_utf8_lossy(&attr.value).into_owned(),
                            b"type" => cot_type = String::from_utf8_lossy(&attr.value).into_owned(),
                            _ => {}
                        }
                    }
                }
                b"contact" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"callsign" {
                            callsign = String::from_utf8_lossy(&attr.value).into_owned();
                        }
                    }
                }
                b"link" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"point" {
                            let pt_str = String::from_utf8_lossy(&attr.value);
                            let mut parts = pt_str.split(',');
                            if let (Some(lat_s), Some(lon_s)) = (parts.next(), parts.next()) {
                                if let (Ok(lat), Ok(lon)) =
                                    (lat_s.trim().parse::<f64>(), lon_s.trim().parse::<f64>())
                                {
                                    let (e_m, _, n_m) = origin.lla_to_enu(lat, lon, origin.alt_m);
                                    vertices_en.push((e_m, n_m));
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    if uid.is_empty() || vertices_en.len() < 3 {
        return None;
    }
    let name = if callsign.is_empty() {
        uid.clone()
    } else {
        callsign
    };
    let upper = name.to_ascii_uppercase();
    let affiliation = if cot_type.starts_with("a-h")
        || upper.contains("KILL")
        || upper.contains("ROZ")
        || upper.contains("HOSTILE")
        || upper.contains("RED")
    {
        Affiliation::Hostile
    } else {
        Affiliation::Friendly
    };

    Some(TacticalZone {
        uid,
        name,
        affiliation,
        vertices_en,
        floor_m: 0.0,
        ceiling_m: 400.0,
        inside_uids: Vec::new(),
    })
}

/// Evaluate whether a 3D target `(east_m, up_m, north_m)` is inside a `ThreatDome` hemisphere
/// AND has unobstructed terrain line-of-sight from the emitter's radar mast (`REQ-TAC-02`).
pub fn is_engaged_by_threat_dome(
    dome: &ThreatDome,
    target_enu: (f64, f64, f64),
    provider: &dyn ElevationProvider,
) -> bool {
    let (de, du, dn) = (
        target_enu.0 - dome.east_m,
        target_enu.1 - dome.up_m,
        target_enu.2 - dome.north_m,
    );
    if du < -15.0 || du > dome.ceiling_m {
        return false;
    }
    if (de * de + du * du + dn * dn).sqrt() > dome.radius_m {
        return false;
    }
    let radar_mast_enu = (dome.east_m, dome.up_m + 12.0, dome.north_m);
    compute_link_los(provider, radar_mast_enu, target_enu, 28).clear
}

/// Jordan Curve Theorem (even-odd ray-casting) 2D point-in-polygon check in `(east_m, north_m)` (`REQ-TAC-03`).
pub fn point_in_polygon_en(east_m: f64, north_m: f64, vertices: &[(f64, f64)]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let (ei, ni) = vertices[i];
        let (ej, nj) = vertices[j];
        if ((ni > north_m) != (nj > north_m))
            && (east_m < (ej - ei) * (north_m - ni) / (nj - ni) + ei)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Evaluate whether a 3D coordinate `(east_m, up_m, north_m)` lies inside a 3D `TacticalZone` prism (`REQ-TAC-03`).
pub fn is_inside_tactical_zone(zone: &TacticalZone, point_enu: (f64, f64, f64)) -> bool {
    point_enu.1 >= zone.floor_m
        && point_enu.1 <= zone.ceiling_m
        && point_in_polygon_en(point_enu.0, point_enu.2, &zone.vertices_en)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Affiliation;

    struct CanyonWallTerrain;
    impl ElevationProvider for CanyonWallTerrain {
        fn sample_relief_m(&self, east_m: f64, _north_m: f64) -> f64 {
            if (80.0..=140.0).contains(&east_m) {
                130.0
            } else {
                30.0
            }
        }
    }

    #[test]
    fn test_threat_dome_and_polygonal_geofence() {
        let dome = ThreatDome {
            uid: "HST-EMITTER-02".to_string(),
            callsign: "SA-15".to_string(),
            affiliation: Affiliation::Hostile,
            east_m: 0.0,
            up_m: 30.0,
            north_m: 0.0,
            radius_m: 600.0,
            ceiling_m: 450.0,
            engaged_uids: Vec::new(),
        };
        assert!(is_engaged_by_threat_dome(
            &dome,
            (250.0, 320.0, 0.0),
            &CanyonWallTerrain
        ));
        assert!(!is_engaged_by_threat_dome(
            &dome,
            (250.0, 110.0, 0.0),
            &CanyonWallTerrain
        ));

        let zone = TacticalZone {
            uid: "ROZ-1".to_string(),
            name: "ROZ".to_string(),
            affiliation: Affiliation::Hostile,
            vertices_en: vec![
                (-200.0, -200.0),
                (200.0, -200.0),
                (200.0, 200.0),
                (-200.0, 200.0),
            ],
            floor_m: 0.0,
            ceiling_m: 350.0,
            inside_uids: Vec::new(),
        };
        assert!(is_inside_tactical_zone(&zone, (0.0, 180.0, 0.0)));
        assert!(!is_inside_tactical_zone(&zone, (300.0, 180.0, 0.0)));
    }

    #[test]
    fn test_parse_cot_polygon_zone_udf() {
        let origin = GeoOrigin::default();
        let xml = r#"<event version="2.0" uid="ATAK-POLY-01" type="u-d-f" how="h-e"><point lat="39.9200" lon="32.8500" hae="850.0" ce="10" le="10"/><detail><contact callsign="KILLBOX DELTA ROZ"/><link point="39.9180,32.8480,850.0"/><link point="39.9180,32.8520,850.0"/><link point="39.9220,32.8520,850.0"/><link point="39.9220,32.8480,850.0"/></detail></event>"#;
        let zone = parse_cot_polygon_zone(xml, &origin).expect("valid u-d-f polygon zone");
        assert_eq!(zone.uid, "ATAK-POLY-01");
        assert_eq!(zone.affiliation, Affiliation::Hostile);
        assert_eq!(zone.vertices_en.len(), 4);
        assert!(is_inside_tactical_zone(&zone, (0.0, 120.0, 0.0)));
    }
}
