use crate::http::{DynError, HttpRequest, HttpResponse};
use crate::*;
use xrtak_core::{
    build_mapset_manifest, compute_link_los, compute_viewshed_with_provider,
    evaluate_mission_corridor, evaluate_swath_corridor, export_atak_route_cot_xml,
    export_qgc_plan_json, ingest_geotiff_dsm, parse_kml_mission, parse_qgc_plan_mission,
    resolve_tile_with_fallback, resolve_tile_with_placeholder_check, GeoOrigin, ImageryProvider,
    ManifestInput, ProceduralReliefModel, RasterDemProvider, TacticalWorld,
};

pub enum RouteOutput {
    Response(HttpResponse),
    SseStream(broadcast::Receiver<String>),
}

pub async fn dispatch(req: &HttpRequest, state: &GatewayState) -> Result<RouteOutput, DynError> {
    let (method, path) = (req.method.as_str(), req.target.as_str());
    if is_state_changing(method, path) {
        if let Some((code, err)) = check_request_auth(req) {
            return Ok(RouteOutput::Response(HttpResponse::auth_error(code, err)));
        }
    }
    let resp = match (method, path) {
        ("OPTIONS", _) => HttpResponse::no_content(),
        ("GET", "/" | "/index.html") => HttpResponse::html(INDEX_HTML),
        ("GET", "/api/stream") => return Ok(RouteOutput::SseStream(state.tx.subscribe())),
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
        ("POST", "/api/tiles/resolve") => handle_tiles_resolve_post(&req.body, state)?,
        ("POST", "/api/mission/corridor-polygon") => {
            handle_corridor_polygon_post(&req.body, state)?
        }
        ("POST", "/api/mapset/plan") => handle_mapset_plan_post(&req.body, state)?,
        ("POST", "/api/rf/link-budget") => handle_rf_link_budget_post(&req.body, state)?,
        ("POST", "/api/tracks/predict") => handle_tracks_predict_post(&req.body, state)?,
        ("POST", "/api/airspace/deconflict") => handle_deconflict_post(&req.body, state)?,
        ("GET", "/api/export/opord") => handle_export_opord_get(state)?,
        ("POST", "/api/ew/jamming") => handle_ew_jamming_post(&req.body)?,
        ("POST", "/api/medevac/nine-line") => handle_medevac_post(&req.body, state)?,
        ("POST", "/api/fires/ballistics") => handle_ballistics_post(&req.body, state)?,
        ("POST", "/api/uav/endurance") => handle_uav_endurance_post(&req.body)?,
        ("POST", "/api/spawn") => handle_spawn_post(&req.body, state).await?,
        ("POST", "/api/pin/move") => handle_pin_move_post(&req.body, state).await?,
        ("POST", "/api/pin/delete") => handle_pin_delete_post(&req.body, state).await,
        ("GET", "/api/export/geojson") => handle_export_geojson_get(state),
        ("GET", "/api/export/cot") => handle_export_cot_get(state),
        ("POST", "/api/pointer") => handle_pointer_post(&req.body, state).await?,
        ("POST", "/api/sapient") => handle_sapient_post(&req.body, state).await?,
        ("POST", "/api/collab/sync") => handle_collab_sync_post(&req.body, state),
        ("GET", "/api/tak-status") => handle_tak_status_get(state),
        ("POST", "/api/tak-config") => handle_tak_config_post(&req.body, state).await,
        _ => HttpResponse::not_found(),
    };
    Ok(RouteOutput::Response(resp))
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

fn handle_tiles_resolve_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(t_req) = serde_json::from_str::<TileResolveRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_tile_resolve_payload"));
    };
    let origin = lock_world(&state.world).origin;
    let (lat, lon) = (
        t_req.lat.unwrap_or(origin.lat),
        t_req.lon.unwrap_or(origin.lon),
    );
    let provider = ImageryProvider::from_id(t_req.provider.as_deref().unwrap_or("esri"));
    let zoom = t_req.zoom.unwrap_or(15);
    match t_req.rgb_sample.as_deref() {
        Some(s) => HttpResponse::json(&resolve_tile_with_placeholder_check(
            provider,
            lat,
            lon,
            zoom,
            Some(s),
        )),
        None => HttpResponse::json(&resolve_tile_with_fallback(provider, lat, lon, zoom)),
    }
}

fn handle_mapset_plan_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(mp_req) = serde_json::from_str::<MapsetPlanRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_mapset_plan_payload"));
    };
    let origin = lock_world(&state.world).origin;
    let (sw_lat, sw_lon, _) = origin.enu_to_lla(-origin.extent_m, 0.0, -origin.extent_m);
    let (ne_lat, ne_lon, _) = origin.enu_to_lla(origin.extent_m, 0.0, origin.extent_m);
    let name = mp_req.name.unwrap_or_default();
    let manifest = build_mapset_manifest(&ManifestInput {
        name: &name,
        provider: ImageryProvider::from_id(mp_req.provider.as_deref().unwrap_or("bing")),
        south_deg: mp_req.south.unwrap_or(sw_lat),
        west_deg: mp_req.west.unwrap_or(sw_lon),
        north_deg: mp_req.north.unwrap_or(ne_lat),
        east_deg: mp_req.east.unwrap_or(ne_lon),
        min_zoom: mp_req.min_zoom.unwrap_or(12),
        max_zoom: mp_req.max_zoom.unwrap_or(16),
    });
    HttpResponse::json(&manifest)
}

fn handle_rf_link_budget_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(rf_req) = serde_json::from_str::<RfLinkBudgetRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_rf_link_budget_payload"));
    };
    let tx = rf_req.tx_enu.unwrap_or((-600.0, -400.0, 150.0));
    let rx = rf_req.rx_enu.unwrap_or((800.0, 600.0, 180.0));
    let band = match rf_req.band.as_deref().unwrap_or("ism2400") {
        "uhf900" | "900" => RfBand::UHF900,
        "shf5800" | "5800" | "5.8" => RfBand::SHF5800,
        _ => RfBand::ISM2400,
    };
    let steps = rf_req.steps.unwrap_or(32);
    let result = {
        let guard = lock_world(&state.world);
        evaluate_rf_link(tx, rx, band, steps, guard.terrain.as_ref())
    };
    HttpResponse::json(&result)
}

fn handle_tracks_predict_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let json_body = if body.is_empty() { "{}" } else { body };
    let Ok(p_req) = serde_json::from_str::<TrackPredictRequest>(json_body) else {
        return Ok(HttpResponse::bad_request("invalid_tracks_predict_payload"));
    };
    let horizons = p_req.horizons_sec.unwrap_or_else(|| vec![15.0, 30.0, 60.0]);
    let stale_thresh_ms = (p_req.stale_threshold_sec.unwrap_or(5.0) * 1000.0) as u64;
    let now_ms = now_unix_ms();
    let predictions: Vec<TrackPrediction> = lock_world(&state.world)
        .entities
        .values()
        .map(|e| {
            let is_stale = now_ms.saturating_sub(e.updated_unix_ms) >= stale_thresh_ms;
            predict_entity_trajectory_struct(&TrackPredictInput {
                uid: &e.uid,
                callsign: &e.callsign,
                current_enu: (e.east_m, e.north_m, e.up_m),
                speed_mps: e.speed_mps,
                course_deg: e.course_deg,
                is_stale,
                initial_ce_m: 5.0,
                horizons: &horizons,
            })
        })
        .collect();
    HttpResponse::json(&predictions)
}

fn handle_deconflict_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    if !body.is_empty() && serde_json::from_str::<serde_json::Value>(body).is_err() {
        return Ok(HttpResponse::bad_request("invalid_deconflict_payload"));
    }
    let guard = lock_world(&state.world);
    let mut list: Vec<DeconflictEntity<'_>> = guard
        .entities
        .values()
        .filter(|e| e.dimension == xrtak_core::Dimension::Air && e.speed_mps > 1.0)
        .map(|e| {
            (
                e.callsign.as_str(),
                (e.east_m, e.north_m, e.up_m),
                e.speed_mps,
                e.course_deg,
            )
        })
        .collect();
    if list.len() < 2 {
        list = guard
            .entities
            .values()
            .map(|e| {
                (
                    e.callsign.as_str(),
                    (e.east_m, e.north_m, e.up_m),
                    e.speed_mps.max(15.0),
                    e.course_deg,
                )
            })
            .collect();
    }
    HttpResponse::json(&evaluate_theater_deconfliction(&list))
}

fn handle_export_opord_get(state: &GatewayState) -> Result<HttpResponse, DynError> {
    let guard = lock_world(&state.world);
    let units: Vec<OpordUnitSummary> = guard
        .entities
        .values()
        .map(|e| OpordUnitSummary {
            callsign: e.callsign.clone(),
            cot_type: e.cot_type.clone(),
            affiliation: format!("{:?}", e.affiliation),
            lat: e.lat,
            lon: e.lon,
            alt_m: e.up_m,
        })
        .collect();
    let origin_lla = (guard.origin.lat, guard.origin.lon, guard.origin.alt_m);
    let bundle = xrtak_core::opord::build_opord_bundle_with_features(
        &format!("XRTAK-{}", guard.terrain_name.to_uppercase()),
        origin_lla,
        &units,
        &guard.tactical_zones,
        &guard.threat_domes,
    );
    HttpResponse::json(&bundle)
}

fn handle_ew_jamming_post(body: &str) -> Result<HttpResponse, DynError> {
    let Ok(ew_req) = serde_json::from_str::<ew_dto::EwJammingRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_ew_jamming_payload"));
    };
    let band = match ew_req.band.as_deref() {
        Some("gnss") | Some("gnss_l1") => xrtak_core::EwTargetBand::GnssL1,
        Some("video") | Some("video_downlink") => xrtak_core::EwTargetBand::VideoDownlink,
        Some("radar") | Some("radar_xband") => xrtak_core::EwTargetBand::RadarXBand,
        _ => xrtak_core::EwTargetBand::TacticalC2,
    };
    let threshold = ew_req.jamming_margin_threshold_db.unwrap_or(6.0);
    let assessment =
        xrtak_core::evaluate_link_jamming(&ew_req.jammer, &ew_req.link, band, threshold);
    let burn_through = ew_req.radar_burn_through.map(|r| {
        let j = &ew_req.jammer;
        let d_rj = (j.east_m.powi(2) + j.north_m.powi(2) + j.up_m.powi(2)).sqrt();
        xrtak_core::compute_burn_through_range(
            "GROUND-RADAR",
            r.radar_power_w,
            r.radar_gain_dbi,
            r.target_rcs_m2,
            &j.callsign,
            j.transmitter_power_w,
            j.antenna_gain_dbi,
            d_rj,
            r.current_target_range_m,
        )
    });
    HttpResponse::json(&ew_dto::EwJammingResponse {
        assessment,
        burn_through,
    })
}

fn handle_medevac_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(m_req) = serde_json::from_str::<medevac_dto::MedevacNineLineRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_medevac_request_payload"));
    };
    let prec = match m_req.precedence.as_deref() {
        Some("urgent") => xrtak_core::PatientPrecedence::Urgent,
        Some("urgent_surgical") => xrtak_core::PatientPrecedence::UrgentSurgical,
        Some("priority") => xrtak_core::PatientPrecedence::Priority,
        Some("routine") => xrtak_core::PatientPrecedence::Routine,
        _ => xrtak_core::PatientPrecedence::Urgent,
    };
    let (e, u, n) = m_req.enu_center;
    let radius = m_req.hlz_radius_m.unwrap_or(25.0);
    let (hlz_suitability, lat, lon) = {
        let guard = lock_world(&state.world);
        let elev: &dyn xrtak_core::ElevationProvider = guard.terrain.as_ref();
        let suit = xrtak_core::evaluate_hlz_suitability(e, n, radius, elev, &guard.threat_domes);
        let (lat, lon, _) = guard.origin.enu_to_lla(e, u, n);
        (suit, lat, lon)
    };
    let report = xrtak_core::MedevacReport {
        line1_location: m_req.location,
        line2_freq_callsign: m_req.freq_callsign,
        line3_precedence: prec,
        line4_special_equipment: m_req
            .special_equipment
            .unwrap_or_else(|| "None / Standard".to_string()),
        line5_litter_count: m_req.litter_count.unwrap_or(1),
        line5_ambulatory_count: m_req.ambulatory_count.unwrap_or(0),
        line6_security: m_req
            .security
            .unwrap_or_else(|| "N - No enemy troops in area".to_string()),
        line7_hlz_marking: m_req
            .marking
            .unwrap_or_else(|| "Panels / Smoke Green".to_string()),
        line8_patient_nationality: m_req
            .nationality
            .unwrap_or_else(|| "Friendly Military".to_string()),
        line9_nbc_or_terrain: m_req
            .terrain_remarks
            .unwrap_or_else(|| "Valley clearing".to_string()),
        lat,
        lon,
        east_m: e,
        north_m: n,
        up_m: u,
    };
    let formatted_nine_line = xrtak_core::format_nato_nine_line_text(&report);
    HttpResponse::json(&medevac_dto::MedevacNineLineResponse {
        formatted_nine_line,
        report,
        hlz_suitability,
    })
}

fn handle_ballistics_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(b_req) = serde_json::from_str::<fires_dto::BallisticsRequest>(body) else {
        return Ok(HttpResponse::bad_request(
            "invalid_ballistics_request_payload",
        ));
    };
    let munition = match b_req.munition.as_deref() {
        Some("mortar") | Some("mortar120") => xrtak_core::MunitionType::Mortar120mm,
        Some("loitering") | Some("drone") => xrtak_core::MunitionType::LoiteringMunition,
        Some("rocket") | Some("mlrs") => xrtak_core::MunitionType::TacticalRocket,
        _ => xrtak_core::MunitionType::Artillery155mm,
    };
    let samples = b_req.samples.unwrap_or(32);
    let solution = {
        let guard = lock_world(&state.world);
        xrtak_core::compute_ballistic_trajectory(
            b_req.origin_enu,
            b_req.target_enu,
            munition,
            b_req.elevation_angle_deg,
            guard.terrain.as_ref(),
            samples,
        )
    };
    HttpResponse::json(&fires_dto::BallisticsResponse { solution })
}

fn handle_uav_endurance_post(body: &str) -> Result<HttpResponse, DynError> {
    let Ok(bat_req) = serde_json::from_str::<battery_dto::UavEnduranceRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_uav_endurance_payload"));
    };
    let wind = bat_req.wind.unwrap_or_default();
    let steps = bat_req.radial_steps.unwrap_or(32);
    let solution = xrtak_core::compute_uav_endurance(&bat_req.state, &wind, steps);
    HttpResponse::json(&battery_dto::UavEnduranceResponse { solution })
}

async fn handle_spawn_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(spawn) = serde_json::from_str::<SpawnRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_spawn_payload"));
    };
    let (xml, ent, pins_json) = {
        let mut guard = lock_world(&state.world);
        let ground_h = guard.sample_relief_m(spawn.east_m, spawn.north_m);
        let up_m = ground_h + spawn.alt_offset_m.unwrap_or(0.0);
        let (lat, lon, hae) = guard.origin.enu_to_lla(spawn.east_m, up_m, spawn.north_m);
        let uid = spawn.uid.unwrap_or_else(|| {
            format!(
                "PIN-{}-{}",
                spawn.callsign.replace(' ', "-"),
                now_unix_ms() % 100000
            )
        });
        let remarks = spawn.remarks.unwrap_or_else(|| {
            format!(
                "Pinned via XR-TAK at {:.5}N {:.5}E ({:.0}m ASL)",
                lat, lon, hae
            )
        });
        let xml = format!(
            r#"<event version="2.0" uid="{uid}" type="{cot_type}" how="h-g-i-g-o"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae:.1}" ce="5" le="5"/><detail><contact callsign="{callsign}"/><track course="0.0" speed="0.0"/><remarks>{remarks}</remarks></detail></event>"#,
            uid = escape_cot_attr(&uid),
            cot_type = escape_cot_attr(&spawn.cot_type),
            callsign = escape_cot_attr(&spawn.callsign),
            remarks = escape_cot_attr(&remarks)
        );
        let r = guard.ingest_cot_xml(&xml, now_unix_ms());
        let pj = if r.is_ok() {
            OPERATOR_PINS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(uid);
            snapshot_persistent_pins_json(&guard)
        } else {
            None
        };
        (xml, r, pj)
    };
    flush_persistent_pins(pins_json).await;
    state.send_feedback(xml.as_bytes()).await;
    match ent {
        Ok(entity) => HttpResponse::json(&entity),
        Err(_) => Ok(HttpResponse::bad_request("invalid_spawn_payload")),
    }
}

async fn handle_pin_move_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(mv) = serde_json::from_str::<PinMoveRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_pin_move_payload"));
    };
    let (updated, pins_json) = {
        let mut guard = lock_world(&state.world);
        let up_m = guard.sample_relief_m(mv.east_m, mv.north_m) + mv.alt_offset_m.unwrap_or(0.0);
        let (lat, lon, hae) = guard.origin.enu_to_lla(mv.east_m, up_m, mv.north_m);
        let out_ent = guard.entities.get_mut(&mv.uid).map(|ent| {
            (ent.east_m, ent.north_m, ent.up_m) = (mv.east_m, mv.north_m, up_m);
            (ent.lat, ent.lon, ent.hae_m) = (lat, lon, hae);
            if let Some(cs) = mv.callsign.filter(|s| !s.trim().is_empty()) {
                ent.callsign = cs;
            }
            if let Some(rmk) = mv.remarks {
                ent.remarks = rmk;
            }
            ent.updated_unix_ms = now_unix_ms();
            let cloned = ent.clone();
            (entity_to_cot_xml(&cloned), cloned)
        });
        let pj = out_ent.is_some().then(|| {
            guard.refresh_threat_domes_and_zones();
            snapshot_persistent_pins_json(&guard)
        });
        (out_ent, pj.flatten())
    };
    flush_persistent_pins(pins_json).await;
    match updated {
        Some((xml, entity)) => {
            state.send_feedback(xml.as_bytes()).await;
            HttpResponse::json(&entity)
        }
        None => Ok(HttpResponse::bad_request("invalid_pin_move_payload")),
    }
}

async fn handle_pin_delete_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(del) = serde_json::from_str::<PinDeleteRequest>(body) else {
        return HttpResponse::bad_request("invalid_pin_delete_payload");
    };
    let (removed, pins_json) = {
        let mut guard = lock_world(&state.world);
        let r = guard.entities.remove(&del.uid).is_some();
        OPERATOR_PINS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&del.uid);
        guard.refresh_threat_domes_and_zones();
        (r, snapshot_persistent_pins_json(&guard))
    };
    flush_persistent_pins(pins_json).await;
    HttpResponse::json_str(serde_json::json!({ "deleted": removed, "uid": del.uid }).to_string())
}

fn handle_export_geojson_get(state: &GatewayState) -> HttpResponse {
    let guard = lock_world(&state.world);
    let mut ents: Vec<&TacticalEntity> = guard.entities.values().collect();
    ents.sort_by(|a, b| a.uid.cmp(&b.uid));
    let features: Vec<serde_json::Value> = ents
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "type": "Feature",
                "id": e.uid,
                "geometry": { "type": "Point", "coordinates": [e.lon, e.lat, e.hae_m] },
                "properties": {
                    "uid": e.uid,
                    "callsign": e.callsign,
                    "cot_type": e.cot_type,
                    "affiliation": e.affiliation,
                    "east_m": e.east_m,
                    "north_m": e.north_m,
                    "up_m": e.up_m,
                    "remarks": e.remarks,
                    "updated_unix_ms": e.updated_unix_ms
                }
            })
        })
        .collect();
    HttpResponse::json_str(
        serde_json::json!({
            "type": "FeatureCollection",
            "origin": guard.origin,
            "features": features
        })
        .to_string(),
    )
}

fn handle_export_cot_get(state: &GatewayState) -> HttpResponse {
    let guard = lock_world(&state.world);
    let mut ents: Vec<&TacticalEntity> = guard.entities.values().collect();
    ents.sort_by(|a, b| a.uid.cmp(&b.uid));
    let events: Vec<String> = ents.into_iter().map(entity_to_cot_xml).collect();
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<events origin_lat=\"{:.6}\" origin_lon=\"{:.6}\">\n  {}\n</events>\n",
        guard.origin.lat,
        guard.origin.lon,
        events.join("\n  ")
    );
    HttpResponse::xml(body)
}

async fn handle_pointer_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(ptr_req) = serde_json::from_str::<LaserPointerRequest>(body) else {
        return Ok(HttpResponse::bad_request("invalid_pointer_payload"));
    };
    let (xml, res) = {
        let mut guard = lock_world(&state.world);
        let up_m = guard.sample_relief_m(ptr_req.east_m, ptr_req.north_m);
        let (lat, lon, hae) = guard
            .origin
            .enu_to_lla(ptr_req.east_m, up_m, ptr_req.north_m);
        let cs = ptr_req.callsign.unwrap_or_else(|| "QUEST3-OP".to_string());
        let xml = build_laser_pointer_cot_xml(&cs, lat, lon, hae);
        let res = guard.ingest_cot_xml(&xml, now_unix_ms());
        (xml, res)
    };
    state.send_feedback(xml.as_bytes()).await;
    match res {
        Ok(entity) => HttpResponse::json(&entity),
        Err(_) => Ok(HttpResponse::bad_request("invalid_pointer_payload")),
    }
}

async fn handle_sapient_post(body: &str, state: &GatewayState) -> Result<HttpResponse, DynError> {
    let Ok(report) = serde_json::from_str::<SapientDetectionReport>(body) else {
        return Ok(HttpResponse::bad_request("invalid_sapient_payload"));
    };
    let res = lock_world(&state.world).ingest_sapient_detection(&report, now_unix_ms());
    match res {
        Ok((xml, entity)) => {
            state.send_feedback(xml.as_bytes()).await;
            HttpResponse::json(&entity)
        }
        Err(_) => Ok(HttpResponse::bad_request("invalid_sapient_payload")),
    }
}

fn handle_collab_sync_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(c_req) = serde_json::from_str::<CollabSyncRequest>(body) else {
        return HttpResponse::bad_request("invalid_collab_sync_payload");
    };
    let mut guard = lock_world(&state.world);
    let peer = OperatorPeer {
        operator_id: c_req.operator_id,
        callsign: c_req.callsign,
        device_mode: c_req
            .device_mode
            .unwrap_or_else(|| "DESKTOP-C2".to_string()),
        head_enu: c_req.head_enu.unwrap_or((0.0, 480.0, -420.0)),
        laser_target_enu: c_req.laser_target_enu,
        ptt_active: c_req.ptt_active.unwrap_or(false),
        color_hex: c_req.color_hex.unwrap_or_else(|| "#00ff88".to_string()),
        updated_unix_ms: now_unix_ms(),
    };
    if !guard.collab.peers.contains_key(&peer.operator_id) && guard.collab.peers.len() >= 64 {
        if let Some(oldest) = guard
            .collab
            .peers
            .values()
            .min_by_key(|p| p.updated_unix_ms)
            .map(|p| p.operator_id.clone())
        {
            guard.collab.peers.remove(&oldest);
        }
    }
    guard
        .collab
        .upsert_peer(peer, c_req.shared_scale, c_req.shared_yaw_deg);
    HttpResponse::json_str(
        serde_json::json!({
            "operators": guard.collab.sorted_peers(),
            "table_pose": guard.collab.table_pose,
        })
        .to_string(),
    )
}

fn handle_tak_status_get(state: &GatewayState) -> HttpResponse {
    let status = state.tak_status_rx.borrow().clone();
    match serde_json::to_string(&status) {
        Ok(j) => HttpResponse::json_str(j),
        Err(_) => HttpResponse::bad_request("failed_to_serialize_tak_status"),
    }
}

async fn handle_tak_config_post(body: &str, state: &GatewayState) -> HttpResponse {
    let Ok(cfg) = serde_json::from_str::<TakConfigRequest>(body) else {
        return HttpResponse::bad_request("invalid_tak_config_payload");
    };
    if let Some(addr) = cfg.tak_tcp_addr.filter(|a| !a.trim().is_empty()) {
        let is_tls = cfg.tls_enabled.unwrap_or(false);
        let opts = tak_client::TakClientOptions {
            target_addr: addr,
            tls_enabled: is_tls,
            ca_cert_pem: cfg.ca_cert_pem,
            client_cert_pem: cfg.client_cert_pem,
            client_key_pem: cfg.client_key_pem,
            sni_hostname: cfg.sni_hostname,
            verify_hostname: cfg.verify_hostname,
            handshake_timeout_ms: cfg.handshake_timeout_ms,
            heartbeat_interval_ms: cfg.heartbeat_interval_ms,
            heartbeat_timeout_ms: cfg.heartbeat_timeout_ms,
            client_uid: cfg.client_uid,
        };
        spawn_tak_client(opts, state);
    }
    let (out, pins_json) = {
        let mut guard = lock_world(&state.world);
        if let Some(sim) = cfg.sim_enabled {
            guard.set_sim_enabled(sim);
        }
        if let Some(auto_rc) = cfg.auto_recenter {
            guard.auto_recenter_on_next_live_cot = auto_rc;
        }
        if let (Some(lat), Some(lon)) = (cfg.origin_lat, cfg.origin_lon) {
            let alt_m = cfg.origin_alt_m.unwrap_or(guard.origin.alt_m);
            let extent_m = guard.origin.extent_m;
            guard.recenter_origin(GeoOrigin {
                lat,
                lon,
                alt_m,
                extent_m,
            });
        }
        let pj = snapshot_persistent_pins_json(&guard);
        let j = serde_json::json!({
            "origin": guard.origin,
            "sim_enabled": guard.sim_enabled,
            "auto_recenter": guard.auto_recenter_on_next_live_cot
        })
        .to_string();
        (j, pj)
    };
    flush_persistent_pins(pins_json).await;
    HttpResponse::json_str(out)
}
