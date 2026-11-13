use crate::geo::ElevationProvider;
use crate::GeoOrigin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMissionWaypoint {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
    /// True when `alt_m` is relative to ground/takeoff (AGL) rather than ASL/HAE
    pub alt_is_relative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionWaypoint3d {
    pub index: usize,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub flight_asl_m: f64,
    pub flight_up_m: f64,
    pub terrain_relief_m: f64,
    pub terrain_asl_m: f64,
    pub agl_m: f64,
    pub cumulative_dist_m: f64,
    pub clearance_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCorridorResult {
    pub mission_name: String,
    pub corridor_width_m: f64,
    pub min_safe_agl_m: f64,
    pub total_distance_m: f64,
    pub min_agl_m: f64,
    pub max_asl_m: f64,
    pub violation_count: usize,
    pub waypoints: Vec<MissionWaypoint3d>,
}

/// Projects raw mission waypoints into the active local tangent plane (`GeoOrigin`),
/// samples terrain elevation via `ElevationProvider`, and evaluates AGL clearance (`REQ-MIS-02`).
pub fn evaluate_mission_corridor(
    mission_name: impl Into<String>,
    raw_waypoints: &[RawMissionWaypoint],
    origin: &GeoOrigin,
    terrain: &dyn ElevationProvider,
    corridor_width_m: f64,
    min_safe_agl_m: f64,
) -> MissionCorridorResult {
    let mut waypoints = Vec::with_capacity(raw_waypoints.len());
    let mut cumulative_dist_m = 0.0;
    let mut min_agl_m = f64::INFINITY;
    let mut max_asl_m = f64::NEG_INFINITY;
    let mut violation_count = 0usize;
    let mut prev_en: Option<(f64, f64)> = None;

    for (idx, wp) in raw_waypoints.iter().enumerate() {
        let (east_m, _, north_m) = origin.lla_to_enu(wp.lat, wp.lon, origin.alt_m);
        let terrain_relief_m = terrain.sample_relief_m(east_m, north_m);
        let terrain_asl_m = origin.alt_m + terrain_relief_m;

        let flight_asl_m = if wp.alt_is_relative {
            terrain_asl_m + wp.alt_m
        } else if wp.alt_m <= 0.0 {
            terrain_asl_m + 60.0
        } else {
            wp.alt_m
        };
        let flight_up_m = (flight_asl_m - origin.alt_m).max(0.0);
        let agl_m = flight_asl_m - terrain_asl_m;

        if let Some((pe, pn)) = prev_en {
            cumulative_dist_m += ((east_m - pe).powi(2) + (north_m - pn).powi(2)).sqrt();
        }
        prev_en = Some((east_m, north_m));

        min_agl_m = min_agl_m.min(agl_m);
        max_asl_m = max_asl_m.max(flight_asl_m);
        let clearance_violation = agl_m < min_safe_agl_m;
        if clearance_violation {
            violation_count += 1;
        }

        waypoints.push(MissionWaypoint3d {
            index: idx,
            name: wp.name.clone(),
            lat: wp.lat,
            lon: wp.lon,
            east_m: (east_m * 100.0).round() / 100.0,
            north_m: (north_m * 100.0).round() / 100.0,
            flight_asl_m: (flight_asl_m * 100.0).round() / 100.0,
            flight_up_m: (flight_up_m * 100.0).round() / 100.0,
            terrain_relief_m: (terrain_relief_m * 100.0).round() / 100.0,
            terrain_asl_m: (terrain_asl_m * 100.0).round() / 100.0,
            agl_m: (agl_m * 100.0).round() / 100.0,
            cumulative_dist_m: (cumulative_dist_m * 10.0).round() / 10.0,
            clearance_violation,
        });
    }

    if waypoints.is_empty() {
        min_agl_m = 0.0;
        max_asl_m = origin.alt_m;
    }

    MissionCorridorResult {
        mission_name: mission_name.into(),
        corridor_width_m: corridor_width_m.max(10.0),
        min_safe_agl_m,
        total_distance_m: (cumulative_dist_m * 10.0).round() / 10.0,
        min_agl_m: (min_agl_m * 100.0).round() / 100.0,
        max_asl_m: (max_asl_m * 100.0).round() / 100.0,
        violation_count,
        waypoints,
    }
}

/// Parses a KML document (`<Placemark><Point><coordinates>` or `<LineString><coordinates>`)
/// into a mission name and ordered `RawMissionWaypoint` list (`REQ-MIS-01`).
pub fn parse_kml_mission(kml: &str) -> Result<(String, Vec<RawMissionWaypoint>), String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(kml);
    reader.config_mut().trim_text(true);

    let mut doc_name = String::from("KML-MISSION");
    let mut in_placemark = false;
    let mut current_pm_name = String::new();
    let mut current_tag = String::new();
    let mut alt_is_relative = false;
    let mut waypoints = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_lowercase();
                if local == "placemark" {
                    in_placemark = true;
                    current_pm_name.clear();
                    alt_is_relative = false;
                }
                current_tag = local;
            }
            Ok(Event::End(e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_lowercase();
                if local == "placemark" {
                    in_placemark = false;
                }
                current_tag.clear();
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().map_err(|e| e.to_string())?.trim().to_string();
                if text.is_empty() {
                    continue;
                }
                if current_tag == "name" {
                    if in_placemark {
                        current_pm_name = text;
                    } else if doc_name == "KML-MISSION" {
                        doc_name = text;
                    }
                } else if current_tag == "altitudemode" {
                    let lower = text.to_lowercase();
                    alt_is_relative = lower.contains("relative");
                } else if current_tag == "coordinates" {
                    let tuples: Vec<&str> = text.split_whitespace().collect();
                    for (idx, tuple) in tuples.iter().enumerate() {
                        let parts: Vec<&str> = tuple.split(',').collect();
                        if parts.len() >= 2 {
                            if let (Ok(lon), Ok(lat)) =
                                (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>())
                            {
                                let alt_m = parts
                                    .get(2)
                                    .and_then(|s| s.trim().parse::<f64>().ok())
                                    .unwrap_or(120.0);
                                let wp_name = if !current_pm_name.is_empty() && tuples.len() == 1 {
                                    current_pm_name.clone()
                                } else {
                                    format!("WP-{}", waypoints.len() + 1 + idx * 0)
                                };
                                waypoints.push(RawMissionWaypoint {
                                    name: wp_name,
                                    lat,
                                    lon,
                                    alt_m,
                                    alt_is_relative,
                                });
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("KML XML parse error: {e}")),
            _ => {}
        }
    }

    if waypoints.is_empty() {
        return Err("No valid <coordinates> waypoints found in KML".to_string());
    }
    Ok((doc_name, waypoints))
}

/// Parses a QGroundControl `.plan` JSON document (`mission.items`) into ordered waypoints (`REQ-MIS-01`).
pub fn parse_qgc_plan_mission(plan_json: &str) -> Result<(String, Vec<RawMissionWaypoint>), String> {
    let v: serde_json::Value =
        serde_json::from_str(plan_json).map_err(|e| format!("Invalid QGC .plan JSON: {e}"))?;
    let items = v
        .get("mission")
        .and_then(|m| m.get("items"))
        .and_then(|i| i.as_array())
        .ok_or_else(|| "QGC .plan missing mission.items array".to_string())?;

    let mut waypoints = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        let params = match item.get("params").and_then(|p| p.as_array()) {
            Some(p) if p.len() >= 7 => p,
            _ => continue,
        };
        let lat = match params[4].as_f64() {
            Some(val) if val.is_finite() && val.abs() > 1e-5 => val,
            _ => continue,
        };
        let lon = match params[5].as_f64() {
            Some(val) if val.is_finite() && val.abs() > 1e-5 => val,
            _ => continue,
        };
        let alt_m = params[6]
            .as_f64()
            .or_else(|| item.get("Altitude").and_then(|a| a.as_f64()))
            .unwrap_or(100.0);
        // QGC frame 3 = MAV_FRAME_GLOBAL_RELATIVE_ALT, AltitudeMode 1 = Relative
        let frame = item.get("frame").and_then(|f| f.as_u64()).unwrap_or(3);
        let alt_mode = item.get("AltitudeMode").and_then(|a| a.as_u64()).unwrap_or(1);
        let alt_is_relative = frame == 3 || alt_mode == 1;

        waypoints.push(RawMissionWaypoint {
            name: format!("QGC-WP-{}", idx + 1),
            lat,
            lon,
            alt_m,
            alt_is_relative,
        });
    }

    if waypoints.is_empty() {
        return Err("No valid lat/lon waypoints found in QGC .plan mission.items".to_string());
    }
    Ok(("QGC-PLAN-MISSION".to_string(), waypoints))
}

/// Exports ENU + AGL waypoints `(east_m, north_m, agl_m)` into a QGroundControl `.plan` v1 JSON document (`REQ-MAV-01`).
pub fn export_qgc_plan_json(
    origin: &GeoOrigin,
    terrain: &dyn ElevationProvider,
    waypoints_en_agl: &[(f64, f64, f64)],
) -> String {
    let mut items = Vec::with_capacity(waypoints_en_agl.len());
    for (idx, &(east_m, north_m, agl_m)) in waypoints_en_agl.iter().enumerate() {
        let relief_m = terrain.sample_relief_m(east_m, north_m);
        let (lat, lon, _) = origin.enu_to_lla(east_m, relief_m + agl_m, north_m);
        let cmd = if idx == 0 { 22 } else { 16 }; // 22 = MAV_CMD_NAV_TAKEOFF, 16 = MAV_CMD_NAV_WAYPOINT
        items.push(serde_json::json!({
            "AMSLAltAboveTerrain": null,
            "Altitude": (agl_m * 10.0).round() / 10.0,
            "AltitudeMode": 1,
            "autoContinue": true,
            "command": cmd,
            "doJumpId": idx + 1,
            "frame": 3,
            "params": [0, 0, 0, null, (lat * 1e7).round() / 1e7, (lon * 1e7).round() / 1e7, (agl_m * 10.0).round() / 10.0],
            "type": "SimpleItem"
        }));
    }

    let home_lla = if let Some(&(e0, n0, _)) = waypoints_en_agl.first() {
        let r0 = terrain.sample_relief_m(e0, n0);
        let (lat0, lon0, hae0) = origin.enu_to_lla(e0, r0, n0);
        [lat0, lon0, hae0]
    } else {
        [origin.lat, origin.lon, origin.alt_m]
    };

    serde_json::to_string_pretty(&serde_json::json!({
        "fileType": "Plan",
        "geoFence": { "circles": [], "polygons": [], "version": 2 },
        "groundStation": "XR-TAK Mixed-Reality C2",
        "mission": {
            "cruiseSpeed": 18,
            "firmwareType": 12,
            "globalPlanAltitudeMode": 1,
            "hoverSpeed": 5,
            "items": items,
            "plannedHomePosition": home_lla,
            "vehicleType": 2,
            "version": 2
        },
        "rallyPoints": { "points": [], "version": 2 },
        "version": 1
    }))
    .unwrap_or_default()
}

/// Exports ENU + AGL waypoints `(east_m, north_m, agl_m)` as an ATAK `b-m-r` (Route) CoT XML packet (`REQ-MAV-01`).
pub fn export_atak_route_cot_xml(
    route_uid: &str,
    route_callsign: &str,
    origin: &GeoOrigin,
    terrain: &dyn ElevationProvider,
    waypoints_en_agl: &[(f64, f64, f64)],
) -> String {
    let (lat0, lon0, hae0) = if let Some(&(e0, n0, agl0)) = waypoints_en_agl.first() {
        let r0 = terrain.sample_relief_m(e0, n0);
        origin.enu_to_lla(e0, r0 + agl0, n0)
    } else {
        (origin.lat, origin.lon, origin.alt_m)
    };

    let mut links = String::new();
    for (idx, &(east_m, north_m, agl_m)) in waypoints_en_agl.iter().enumerate() {
        let relief_m = terrain.sample_relief_m(east_m, north_m);
        let (lat, lon, hae) = origin.enu_to_lla(east_m, relief_m + agl_m, north_m);
        links.push_str(&format!(
            r#"<link uid="{route_uid}-WP{wp}" callsign="WP-{wp}" type="b-m-p-w" point="{lat:.6},{lon:.6},{hae:.1}" relation="c"/>"#,
            wp = idx + 1
        ));
    }

    format!(
        r#"<event version="2.0" uid="{route_uid}" type="b-m-r" how="h-g-i-g-o"><point lat="{lat0:.6}" lon="{lon0:.6}" hae="{hae0:.1}" ce="10.0" le="10.0"/><detail><contact callsign="{route_callsign}"/>{links}<strokeColor value="-256"/>< strokeWeight value="3.0"/><remarks>Exported 3D Mission Route from XR-TAK</remarks></detail></event>"#
    )
    .replace("< strokeWeight", "<strokeWeight")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::RasterDemProvider;

    #[test]
    fn test_kml_mission_parsing_and_agl_clearance() {
        let kml = r#"<?xml version="1.0" encoding="utf-8"?>
        <kml xmlns="http://www.opengis.net/kml/2.2">
          <Document>
            <name>llanbedr_corridor_mission</name>
            <Placemark><name>WP-1</name><Point><coordinates>-4.12877,52.81019,920.0</coordinates></Point></Placemark>
            <Placemark><name>WP-2</name><Point><coordinates>-4.12952,52.81301,865.0</coordinates></Point></Placemark>
          </Document>
        </kml>"#;

        let (name, raw) = parse_kml_mission(kml).expect("valid KML mission");
        assert_eq!(name, "llanbedr_corridor_mission");
        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0].name, "WP-1");
        assert!((raw[0].lat - 52.81019).abs() < 1e-5);
        assert!((raw[0].lon - (-4.12877)).abs() < 1e-5);

        let origin = GeoOrigin {
            lat: 52.81019,
            lon: -4.12877,
            alt_m: 850.0,
            extent_m: 1500.0,
        };
        let dem = RasterDemProvider::new("flat-10m", 2, 2, 1500.0, vec![10.0, 10.0, 10.0, 10.0])
            .expect("flat dem");

        // Terrain ASL = 850 + 10 = 860m.
        // WP-1 flight ASL = 920m -> AGL = 60m (safe >= 30m)
        // WP-2 flight ASL = 865m -> AGL = 5m (violation < 30m!)
        let res = evaluate_mission_corridor(&name, &raw, &origin, &dem, 120.0, 30.0);
        assert_eq!(res.waypoints.len(), 2);
        assert!((res.waypoints[0].agl_m - 60.0).abs() < 0.1);
        assert!(!res.waypoints[0].clearance_violation);
        assert!((res.waypoints[1].agl_m - 5.0).abs() < 0.1);
        assert!(res.waypoints[1].clearance_violation);
        assert_eq!(res.violation_count, 1);
        assert!(res.total_distance_m > 300.0);
    }

    #[test]
    fn test_qgc_plan_parsing_and_relative_altitude() {
        let qgc = r#"{
            "fileType": "Plan",
            "mission": {
                "items": [
                    { "command": 22, "frame": 3, "params": [0, 0, 0, null, 39.9200, 32.8500, 85.0] },
                    { "command": 16, "frame": 3, "params": [0, 0, 0, null, 39.9250, 32.8550, 20.0] }
                ]
            }
        }"#;
        let (name, raw) = parse_qgc_plan_mission(qgc).expect("valid QGC .plan");
        assert_eq!(name, "QGC-PLAN-MISSION");
        assert_eq!(raw.len(), 2);
        assert!(raw[0].alt_is_relative);

        let origin = GeoOrigin::default();
        let dem = RasterDemProvider::new("flat", 2, 2, 1500.0, vec![15.0, 15.0, 15.0, 15.0])
            .expect("flat dem");
        let res = evaluate_mission_corridor(&name, &raw, &origin, &dem, 80.0, 30.0);
        assert!((res.waypoints[0].agl_m - 85.0).abs() < 0.1);
        assert!((res.waypoints[1].agl_m - 20.0).abs() < 0.1);
        assert_eq!(res.violation_count, 1);
    }

    #[test]
    fn test_export_qgc_plan_and_atak_bmr_cot_route() {
        let origin = GeoOrigin::default();
        let dem = RasterDemProvider::new("flat-10m", 2, 2, 1500.0, vec![10.0, 10.0, 10.0, 10.0])
            .expect("flat dem");
        let wps = vec![(-250.0, -180.0, 90.0), (120.0, 240.0, 110.0), (420.0, 310.0, 95.0)];

        let qgc_json = export_qgc_plan_json(&origin, &dem, &wps);
        let (_, reparsed) = parse_qgc_plan_mission(&qgc_json).expect("round-trip QGC .plan");
        assert_eq!(reparsed.len(), 3);
        assert!((reparsed[0].alt_m - 90.0).abs() < 1e-3);
        assert!((reparsed[1].alt_m - 110.0).abs() < 1e-3);

        let bmr_xml = export_atak_route_cot_xml("RTE-XR-01", "ALPHA-CORRIDOR", &origin, &dem, &wps);
        assert!(bmr_xml.contains(r#"type="b-m-r""#));
        assert!(bmr_xml.contains(r#"callsign="ALPHA-CORRIDOR""#));
        assert!(bmr_xml.contains(r#"uid="RTE-XR-01-WP3""#));
    }

    #[test]
    fn test_kml_linestring_waypoint_names_are_sequential() {
        let kml = "<kml><Document><Placemark><name>Route</name><LineString>\
            <coordinates>1.0,2.0,10 1.1,2.1,10 1.2,2.2,10</coordinates>\
            </LineString></Placemark></Document></kml>";
        let (_, raw) = parse_kml_mission(kml).expect("valid KML");
        let names: Vec<&str> = raw.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, ["WP-1", "WP-2", "WP-3"]);
    }
}
