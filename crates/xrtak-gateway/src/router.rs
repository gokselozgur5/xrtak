use crate::http::{DynError, HttpRequest, HttpResponse};
use crate::{
    flush_persistent_pins, lock_world, now_unix_ms, snapshot_persistent_pins_json, GatewayState,
    GeoTiffUploadRequest, SpiSlewRequest, TerrainSwitchRequest, WorldSnapshot, INDEX_HTML,
};
use xrtak_core::{
    ingest_geotiff_dsm, GeoOrigin, ProceduralReliefModel, RasterDemProvider, TacticalWorld,
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
