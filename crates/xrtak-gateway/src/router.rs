use crate::http::{DynError, HttpRequest, HttpResponse};
use crate::{
    flush_persistent_pins, lock_world, now_unix_ms, snapshot_persistent_pins_json,
    CorridorPolygonRequest, ElevationProfileRequest, GatewayState, GeoTiffUploadRequest,
    LinkLosRequest, MissionExportRequest, MissionImportRequest, SpiSlewRequest,
    TerrainSwitchRequest, ViewshedRequest, WorldSnapshot, INDEX_HTML,
};
use xrtak_core::{
    compute_link_los, compute_viewshed_with_provider, evaluate_mission_corridor,
    evaluate_swath_corridor, export_atak_route_cot_xml, export_qgc_plan_json, ingest_geotiff_dsm,
    parse_kml_mission, parse_qgc_plan_mission, GeoOrigin, ProceduralReliefModel, RasterDemProvider,
    TacticalWorld,
};

pub async fn try_dispatch(
    req: &HttpRequest,
    state: &GatewayState,
) -> Result<Option<HttpResponse>, DynError> {
    let (method, path) = (req.method.as_str(), req.target.as_str());
    let resp = match (method, path) {
        ("OPTIONS", _) => HttpResponse::no_content(),
        ("GET", "/" | "/index.html") => HttpResponse::html(INDEX_HTML),
        ("GET", "/api/state") => {
            let snap = WorldSnapshot::from_world(&lock_world(&state.world));
            HttpResponse::json(&snap)?
        }
        ("GET", p) if p.starts_with("/api/aar") => handle_aar(p, state),
        ("GET", p) if p.starts_with("/api/hud") => handle_hud(p, state),
        ("GET", "/api/terrain") => terrain_grid_response(&lock_world(&state.world)),
        ("POST", "/api/terrain") => handle_terrain_post(&req.body, state),
        ("POST", "/api/terrain/geotiff") => handle_geotiff_post(&req.body, state),
        ("POST", "/api/cot") => handle_cot_post(&req.body, state).await?,
        ("POST", "/api/spi") => handle_spi_post(&req.body, state)?,
        ("POST", "/api/viewshed") => handle_viewshed_post(&req.body, state)?,
        ("POST", "/api/los") => handle_los_post(&req.body, state)?,
        ("POST", "/api/elevation-profile") => handle_elevation_profile_post(&req.body, state)?,
        ("POST", "/api/mission/import") => handle_mission_import_post(&req.body, state),
        ("POST", "/api/mission/export") => handle_mission_export_post(&req.body, state).await,
        ("POST", "/api/mission/corridor-polygon") => {
            handle_corridor_polygon_post(&req.body, state)?
        }
        _ => return Ok(None),
    };
    Ok(Some(resp))
}

fn handle_aar(path: &str, state: &GatewayState) -> HttpResponse {
    let target_t: Option<f64> = path
        .split("t=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .and_then(|s| s.parse::<f64>().ok());
    let guard = lock_world(&state.world);
    let summary = guard.aar.summary();
    let frame = guard
        .aar
        .sample_at(target_t.unwrap_or(summary.max_time_sec));
    let (domes, zones) = frame
        .as_ref()
        .map(|f| guard.evaluate_overlays_for_entities(&f.entities))
        .unwrap_or_default();
    HttpResponse::json_str(
        serde_json::json!({
            "summary": summary,
            "frame": frame,
            "threat_domes": domes,
            "tactical_zones": zones
        })
        .to_string(),
    )
}

fn handle_hud(path: &str, state: &GatewayState) -> HttpResponse {
    let obs = path
        .split("observer=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap_or("GND-SPEAR-ALPHA");
    let cues = lock_world(&state.world).compute_visor_hud_cues(obs);
    let items: Vec<_> = cues
        .into_iter()
        .map(|(uid, callsign, affiliation, cue)| {
            serde_json::json!({
                "uid": uid,
                "callsign": callsign,
                "affiliation": affiliation,
                "bearing_deg": cue.bearing_deg,
                "slant_range_m": cue.slant_range_m,
                "elevation_deg": cue.elevation_deg
            })
        })
        .collect();
    HttpResponse::json_str(serde_json::json!({ "observer_uid": obs, "cues": items }).to_string())
}

fn terrain_grid_response(world: &TacticalWorld) -> HttpResponse {
    let res = 65usize;
    let grid = world.terrain.export_grid(world.origin.extent_m, res);
    HttpResponse::json_str(
        serde_json::json!({
            "name": world.terrain_name,
            "resolution": res,
            "extent_m": world.origin.extent_m,
            "heights_m": grid
        })
        .to_string(),
    )
}

fn handle_terrain_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(t_req) = serde_json::from_str::<TerrainSwitchRequest>(body) else {
        return HttpResponse::bad_request("invalid_terrain_payload");
    };
    let mut guard = lock_world(&state.world);
    let extent = t_req.extent_m.unwrap_or(guard.origin.extent_m);
    if let (Some(w), Some(h), Some(heights)) = (t_req.width, t_req.height, t_req.heights_m) {
        let name = t_req.preset.unwrap_or_else(|| "custom_dem".to_string());
        match RasterDemProvider::new(&name, w, h, extent, heights) {
            Ok(dem) => guard.set_terrain_provider(&name, Box::new(dem)),
            Err(err) => return HttpResponse::bad_request(&err),
        }
    } else {
        let preset = t_req.preset.unwrap_or_else(|| "valley".to_string());
        if preset == "valley" {
            guard.set_terrain_provider("valley", Box::new(ProceduralReliefModel));
        } else {
            let dem = RasterDemProvider::from_preset(&preset, extent, 65);
            guard.set_terrain_provider(&preset, Box::new(dem));
        }
    }
    terrain_grid_response(&guard)
}

fn handle_geotiff_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(gt_req) = serde_json::from_str::<GeoTiffUploadRequest>(body) else {
        return HttpResponse::bad_request("invalid_geotiff_payload");
    };
    let name = gt_req.name.unwrap_or_else(|| "GEOTIFF-DSM".to_string());
    let extent = lock_world(&state.world).origin.extent_m;
    let parsed = match ingest_geotiff_dsm(&name, extent, 65, &gt_req.bytes) {
        Ok(p) => p,
        Err(err) => return HttpResponse::bad_request(&err),
    };
    let mut guard = lock_world(&state.world);
    let dem_extent = parsed.dem.extent_m;
    if gt_req.auto_recenter.unwrap_or(true) {
        if let Some((clat, clon)) = parsed.center_lat_lon {
            guard.recenter_origin(GeoOrigin {
                lat: clat,
                lon: clon,
                alt_m: parsed.min_asl_m.round(),
                extent_m: dem_extent,
            });
        } else {
            guard.origin.alt_m = parsed.min_asl_m.round();
            guard.origin.extent_m = dem_extent;
        }
    } else {
        guard.origin.extent_m = dem_extent;
    }
    guard.set_terrain_provider(&name, Box::new(parsed.dem));
    let grid = guard.terrain.export_grid(guard.origin.extent_m, 65);
    HttpResponse::json_str(
        serde_json::json!({
            "name": guard.terrain_name,
            "origin": guard.origin,
            "metadata": parsed.metadata,
            "min_asl_m": parsed.min_asl_m,
            "max_asl_m": parsed.max_asl_m,
            "resolution": 65,
            "extent_m": guard.origin.extent_m,
            "heights_m": grid
        })
        .to_string(),
    )
}

async fn handle_cot_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let (res, pins_json) = {
        let mut guard = lock_world(&state.world);
        let r = guard.ingest_cot_xml(body, now_unix_ms());
        let pj = r
            .is_ok()
            .then(|| snapshot_persistent_pins_json(&guard))
            .flatten();
        (r, pj)
    };
    flush_persistent_pins(pins_json).await;
    match res {
        Ok(ent) => HttpResponse::json(&ent),
        Err(err) => Ok(HttpResponse::bad_request(&err)),
    }
}

fn handle_spi_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(spi_req) = serde_json::from_str::<SpiSlewRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_spi_payload"));
    };
    let sensor_uid = spi_req
        .sensor_uid
        .unwrap_or_else(|| "UAV-FALCON-01".to_string());
    if spi_req.clear.unwrap_or(false) {
        lock_world(&state.world).clear_spi_lock(&sensor_uid);
        let j = serde_json::json!({ "status": "cleared", "sensor_uid": sensor_uid }).to_string();
        return Ok(HttpResponse::json_str(j));
    }
    let (east_m, north_m) = (
        spi_req.east_m.unwrap_or(0.0),
        spi_req.north_m.unwrap_or(0.0),
    );
    let res =
        lock_world(&state.world).slew_sensor_to_spi(&sensor_uid, east_m, north_m, now_unix_ms());
    match res {
        Ok(entity) => HttpResponse::json(&entity),
        Err(_) => Ok(HttpResponse::bad_request("invalid_spi_payload")),
    }
}

fn handle_viewshed_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(vs_req) = serde_json::from_str::<ViewshedRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_viewshed_payload"));
    };
    let guard = lock_world(&state.world);
    let vs = compute_viewshed_with_provider(
        guard.terrain.as_ref(),
        vs_req.east_m,
        vs_req.north_m,
        vs_req.height_offset_m.unwrap_or(18.0),
        vs_req.radius_m.unwrap_or(680.0),
        72,
        28,
    );
    HttpResponse::json(&vs)
}

fn handle_los_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(los_req) = serde_json::from_str::<LinkLosRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_los_payload"));
    };
    let guard = lock_world(&state.world);
    let endpoint = |enu: Option<(f64, f64, f64)>, uid: Option<&String>| {
        enu.or_else(|| {
            uid.and_then(|u| guard.entities.get(u))
                .map(|e| (e.east_m, e.up_m + 3.0, e.north_m))
        })
    };
    match (
        endpoint(los_req.from_enu, los_req.from_uid.as_ref()),
        endpoint(los_req.to_enu, los_req.to_uid.as_ref()),
    ) {
        (Some(a), Some(b)) => {
            HttpResponse::json(&compute_link_los(guard.terrain.as_ref(), a, b, 48))
        }
        _ => Ok(HttpResponse::bad_request("invalid_los_payload")),
    }
}

fn handle_elevation_profile_post(
    body: &str,
    state: &GatewayState,
) -> Result<HttpResponse, DynError> {
    match serde_json::from_str::<ElevationProfileRequest>(body) {
        Ok(prof_req) => {
            let prof = lock_world(&state.world).compute_elevation_profile(&prof_req.waypoints_en);
            HttpResponse::json(&prof)
        }
        Err(err) => Ok(HttpResponse::bad_request(&err.to_string())),
    }
}

fn handle_mission_import_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(m_req) = serde_json::from_str::<MissionImportRequest>(body) else {
        return HttpResponse::bad_request("invalid_mission_import_payload");
    };
    let trimmed = m_req.content.trim();
    let is_qgc = m_req
        .format
        .as_deref()
        .map(|f| f.eq_ignore_ascii_case("qgc") || f.eq_ignore_ascii_case("plan"))
        .unwrap_or_else(|| trimmed.starts_with('{'));
    let parsed = if is_qgc {
        parse_qgc_plan_mission(trimmed)
    } else {
        parse_kml_mission(trimmed)
    };
    let (name, raw_wps) = match parsed {
        Ok(v) => v,
        Err(err) => return HttpResponse::bad_request(&err),
    };
    let mut guard = lock_world(&state.world);
    if m_req.auto_recenter.unwrap_or(true) && !raw_wps.is_empty() {
        let n = raw_wps.len() as f64;
        let avg_lat = raw_wps.iter().map(|w| w.lat).sum::<f64>() / n;
        let avg_lon = raw_wps.iter().map(|w| w.lon).sum::<f64>() / n;
        let (alt_m, extent_m) = (guard.origin.alt_m, guard.origin.extent_m);
        guard.recenter_origin(GeoOrigin {
            lat: avg_lat,
            lon: avg_lon,
            alt_m,
            extent_m,
        });
    }
    let corridor = evaluate_mission_corridor(
        &name,
        &raw_wps,
        &guard.origin,
        guard.terrain.as_ref(),
        m_req.corridor_width_m.unwrap_or(120.0),
        m_req.min_safe_agl_m.unwrap_or(35.0),
    );
    let wps_en: Vec<_> = corridor
        .waypoints
        .iter()
        .map(|w| (w.east_m, w.north_m))
        .collect();
    let profile = guard.compute_elevation_profile(&wps_en);
    HttpResponse::json_str(
        serde_json::json!({ "origin": guard.origin, "corridor": corridor, "profile": profile })
            .to_string(),
    )
}

async fn handle_mission_export_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(ex_req) = serde_json::from_str::<MissionExportRequest>(body) else {
        return HttpResponse::bad_request("invalid_mission_export_payload");
    };
    let uid = ex_req.route_uid.as_deref().unwrap_or("RTE-XR-TAK-01");
    let cs = ex_req.route_callsign.as_deref().unwrap_or("XR-CORRIDOR-01");
    let (qgc_plan_json, atak_bmr_xml) = {
        let guard = lock_world(&state.world);
        let qgc = export_qgc_plan_json(
            &guard.origin,
            guard.terrain.as_ref(),
            &ex_req.waypoints_en_agl,
        );
        let bmr = export_atak_route_cot_xml(
            uid,
            cs,
            &guard.origin,
            guard.terrain.as_ref(),
            &ex_req.waypoints_en_agl,
        );
        (qgc, bmr)
    };
    state.send_feedback(atak_bmr_xml.as_bytes()).await;
    let qgc_val: serde_json::Value = serde_json::from_str(&qgc_plan_json).unwrap_or_default();
    HttpResponse::json_str(
        serde_json::json!({
            "route_uid": uid,
            "route_callsign": cs,
            "waypoint_count": ex_req.waypoints_en_agl.len(),
            "qgc_plan": qgc_val,
            "qgc_plan_json": qgc_plan_json,
            "atak_bmr_cot_xml": atak_bmr_xml,
        })
        .to_string(),
    )
}

fn handle_corridor_polygon_post(
    body: &str,
    state: &GatewayState,
) -> Result<HttpResponse, DynError> {
    let Ok(c_req) = serde_json::from_str::<CorridorPolygonRequest>(body) else {
        return Ok(HttpResponse::bad_request(
            "invalid_corridor_polygon_payload",
        ));
    };
    let origin = lock_world(&state.world).origin;
    let name = c_req
        .name
        .unwrap_or_else(|| "XR-SWATH-CORRIDOR".to_string());
    let res = evaluate_swath_corridor(
        name,
        &origin,
        &c_req.waypoints_en,
        c_req.rdp_epsilon_m.unwrap_or(8.0),
        c_req.agl_m.unwrap_or(140.0),
        c_req.hfov_deg,
        c_req.focal_px,
        c_req.sensor_width_px,
    );
    HttpResponse::json(&res)
}
