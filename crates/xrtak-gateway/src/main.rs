use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::broadcast;
use xrtak_core::{
    build_laser_pointer_cot_xml, build_mapset_manifest, build_opord_bundle, compute_link_los,
    compute_viewshed_with_provider, evaluate_mission_corridor, evaluate_rf_link,
    evaluate_swath_corridor, evaluate_theater_deconfliction, export_atak_route_cot_xml,
    export_qgc_plan_json, ingest_geotiff_dsm, parse_kml_mission, parse_qgc_plan_mission,
    predict_entity_trajectory, resolve_tile_with_fallback, AarSummary, GeoOrigin, ImageryProvider,
    OperatorPeer, OpordUnitSummary, ProceduralReliefModel, RasterDemProvider, RfBand,
    SapientDetectionReport, SharedTablePose, TacticalEntity, TacticalWorld, TacticalZone,
    ThreatDome, TrackPredictRequest, TrackPrediction,
};

mod battery_dto;
mod ew_dto;
mod fires_dto;
mod medevac_dto;

const INDEX_HTML: &str = include_str!("../static/index.html");
const CORS_HEADERS: &str = "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nAccess-Control-Allow-Private-Network: true\r\n";

fn json_ok_header(len: usize) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {len}\r\nConnection: close\r\n\r\n"
    )
}

/// Entity id, (east, north, up) position, speed in m/s and course in degrees.
type DeconflictEntity<'a> = (&'a str, (f64, f64, f64), f64, f64);

#[derive(Debug, Clone, Serialize)]
struct WorldSnapshot {
    origin: GeoOrigin,
    terrain_name: String,
    sim_enabled: bool,
    sim_time_sec: f64,
    aar: AarSummary,
    entities: Vec<TacticalEntity>,
    threat_domes: Vec<ThreatDome>,
    tactical_zones: Vec<TacticalZone>,
    operators: Vec<OperatorPeer>,
    table_pose: SharedTablePose,
}

#[derive(Debug, Deserialize)]
struct CollabSyncRequest {
    operator_id: String,
    callsign: String,
    device_mode: Option<String>,
    head_enu: Option<(f64, f64, f64)>,
    laser_target_enu: Option<(f64, f64, f64)>,
    ptt_active: Option<bool>,
    color_hex: Option<String>,
    shared_scale: Option<f64>,
    shared_yaw_deg: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TakConfigRequest {
    sim_enabled: Option<bool>,
    auto_recenter: Option<bool>,
    origin_lat: Option<f64>,
    origin_lon: Option<f64>,
    origin_alt_m: Option<f64>,
    tak_tcp_addr: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LaserPointerRequest {
    callsign: Option<String>,
    east_m: f64,
    north_m: f64,
}

#[derive(Debug, Deserialize)]
struct LinkLosRequest {
    from_uid: Option<String>,
    to_uid: Option<String>,
    from_enu: Option<(f64, f64, f64)>,
    to_enu: Option<(f64, f64, f64)>,
}

#[derive(Debug, Deserialize)]
struct TerrainSwitchRequest {
    preset: Option<String>,
    width: Option<usize>,
    height: Option<usize>,
    extent_m: Option<f64>,
    heights_m: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
struct GeoTiffUploadRequest {
    name: Option<String>,
    bytes: Vec<u8>,
    auto_recenter: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SpawnRequest {
    uid: Option<String>,
    callsign: String,
    cot_type: String,
    east_m: f64,
    north_m: f64,
    alt_offset_m: Option<f64>,
    remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PinMoveRequest {
    uid: String,
    east_m: f64,
    north_m: f64,
    alt_offset_m: Option<f64>,
    callsign: Option<String>,
    remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PinDeleteRequest {
    uid: String,
}

#[derive(Debug, Deserialize)]
struct ElevationProfileRequest {
    waypoints_en: Vec<(f64, f64)>,
}

#[derive(Debug, Deserialize)]
struct MissionImportRequest {
    format: Option<String>,
    content: String,
    corridor_width_m: Option<f64>,
    min_safe_agl_m: Option<f64>,
    auto_recenter: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MissionExportRequest {
    route_uid: Option<String>,
    route_callsign: Option<String>,
    waypoints_en_agl: Vec<(f64, f64, f64)>,
}

#[derive(Debug, Deserialize)]
struct TileResolveRequest {
    provider: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    zoom: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct CorridorPolygonRequest {
    name: Option<String>,
    waypoints_en: Vec<(f64, f64)>,
    rdp_epsilon_m: Option<f64>,
    agl_m: Option<f64>,
    hfov_deg: Option<f64>,
    focal_px: Option<f64>,
    sensor_width_px: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MapsetPlanRequest {
    name: Option<String>,
    provider: Option<String>,
    south: Option<f64>,
    west: Option<f64>,
    north: Option<f64>,
    east: Option<f64>,
    min_zoom: Option<u8>,
    max_zoom: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct RfLinkBudgetRequest {
    tx_enu: Option<(f64, f64, f64)>,
    rx_enu: Option<(f64, f64, f64)>,
    band: Option<String>,
    steps: Option<usize>,
}

const PINS_STORE_PATH: &str = "data/real_pins.json";

fn is_simulated_uid(uid: &str) -> bool {
    matches!(
        uid,
        "UAV-FALCON-01"
            | "UAV-FALCON-01.SPI1"
            | "UAV-VALKYRIE-02"
            | "GND-SPEAR-ALPHA"
            | "GND-OBSERVE-BRAVO"
            | "HST-ARMOR-01"
            | "HST-EMITTER-02"
            | "UNK-BOGEY-07"
    )
}

fn save_persistent_pins(world: &TacticalWorld) {
    let _ = std::fs::create_dir_all("data");
    let mut pins: Vec<&TacticalEntity> = world
        .entities
        .values()
        .filter(|e| !is_simulated_uid(&e.uid) && !e.cot_type.starts_with("b-m-p-c"))
        .collect();
    pins.sort_by(|a, b| a.uid.cmp(&b.uid));
    if let Ok(json) = serde_json::to_string_pretty(&pins) {
        let _ = std::fs::write(PINS_STORE_PATH, json);
    }
}

fn load_persistent_pins(world: &mut TacticalWorld) {
    if let Ok(raw) = std::fs::read_to_string(PINS_STORE_PATH) {
        if let Ok(pins) = serde_json::from_str::<Vec<TacticalEntity>>(&raw) {
            for mut p in pins {
                let (e, u, n) = world.origin.lla_to_enu(p.lat, p.lon, p.hae_m);
                p.east_m = e;
                p.up_m = u.max(world.sample_relief_m(e, n));
                p.north_m = n;
                world.entities.insert(p.uid.clone(), p);
            }
            world.refresh_threat_domes_and_zones();
        }
    }
}

fn entity_to_cot_xml(ent: &TacticalEntity) -> String {
    format!(
        r#"<event version="2.0" uid="{uid}" type="{cot_type}" how="h-g-i-g-o"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae:.1}" ce="5.0" le="5.0"/><detail><contact callsign="{callsign}"/><track course="{course:.1}" speed="{speed:.1}"/><remarks>{remarks}</remarks></detail></event>"#,
        uid = ent.uid,
        cot_type = ent.cot_type,
        lat = ent.lat,
        lon = ent.lon,
        hae = ent.hae_m,
        callsign = ent.callsign,
        course = ent.course_deg,
        speed = ent.speed_mps,
        remarks = ent.remarks
    )
}

#[derive(Debug, Deserialize)]
struct SpiSlewRequest {
    sensor_uid: Option<String>,
    east_m: Option<f64>,
    north_m: Option<f64>,
    clear: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ViewshedRequest {
    east_m: f64,
    north_m: f64,
    height_offset_m: Option<f64>,
    radius_m: Option<f64>,
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_addr_str =
        std::env::var("XRTAK_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8090".to_string());
    let udp_addr_str =
        std::env::var("XRTAK_UDP_ADDR").unwrap_or_else(|_| "0.0.0.0:4242".to_string());
    let http_addr: SocketAddr = http_addr_str.parse()?;
    let udp_cot_addr: SocketAddr = udp_addr_str.parse()?;

    let world = Arc::new(Mutex::new({
        let mut w = TacticalWorld::new(GeoOrigin::default());
        load_persistent_pins(&mut w);
        w
    }));
    let (tx, _rx) = broadcast::channel::<String>(64);

    // 1. Background UDP CoT XML Listener on :4242
    {
        let world_udp = Arc::clone(&world);
        tokio::spawn(async move {
            match UdpSocket::bind(udp_cot_addr).await {
                Ok(sock) => {
                    println!("[XR-TAK] CoT UDP XML Listener active on udp://{udp_cot_addr}");
                    let mut buf = vec![0u8; 65535];
                    loop {
                        if let Ok((len, peer)) = sock.recv_from(&mut buf).await {
                            let xml = String::from_utf8_lossy(&buf[..len]);
                            let mut guard = world_udp.lock().unwrap();
                            match guard.ingest_cot_xml(&xml, now_unix_ms()) {
                                Ok(ent) => println!(
                                    "[XR-TAK] Ingested UDP CoT from {peer}: uid={} callsign={} ({})",
                                    ent.uid, ent.callsign, ent.cot_type
                                ),
                                Err(e) => eprintln!("[XR-TAK] Invalid UDP CoT from {peer}: {e}"),
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[XR-TAK] Could not bind UDP {udp_cot_addr}: {e}"),
            }
        });
    }

    // 2. 20 Hz Tactical World Simulation & Telemetry Broadcast Loop
    {
        let world_sim = Arc::clone(&world);
        let tx_sim = tx.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(50));
            loop {
                ticker.tick().await;
                let payload = {
                    let mut guard = world_sim.lock().unwrap();
                    let now_ms = now_unix_ms();
                    guard.step_simulation(0.05, now_ms);
                    guard.collab.prune_stale(now_ms, 15_000);
                    let mut entities: Vec<TacticalEntity> =
                        guard.entities.values().cloned().collect();
                    entities.sort_by(|a, b| a.uid.cmp(&b.uid));
                    let snap = WorldSnapshot {
                        origin: guard.origin,
                        terrain_name: guard.terrain_name.clone(),
                        sim_enabled: guard.sim_enabled,
                        sim_time_sec: guard.sim_time_sec,
                        aar: guard.aar.summary(),
                        entities,
                        threat_domes: guard.threat_domes.clone(),
                        tactical_zones: guard.tactical_zones.clone(),
                        operators: guard.collab.sorted_peers(),
                        table_pose: guard.collab.table_pose,
                    };
                    serde_json::to_string(&snap).unwrap_or_default()
                };
                let _ = tx_sim.send(payload);
            }
        });
    }

    // 3. HTTP + SSE + CoT/SPI/Viewshed Server on :8090 (with automatic fallback to :8091)
    let listener = match TcpListener::bind(http_addr).await {
        Ok(l) => l,
        Err(_) => TcpListener::bind("0.0.0.0:8091").await?,
    };
    let bound_port = listener.local_addr()?.port();
    println!("==============================================================");
    println!("  XR-TAK — Rust CoT Core + WebXR 3D Tactical Table");
    println!("  WebXR / Browser UI : http://localhost:{bound_port}");
    println!("  Raw CoT UDP Input  : udp://{udp_cot_addr}");
    println!("==============================================================");

    loop {
        let (stream, _) = listener.accept().await?;
        let world_http = Arc::clone(&world);
        let tx_http = tx.clone();
        tokio::spawn(async move {
            let _ = handle_http_connection(stream, world_http, tx_http).await;
        });
    }
}

async fn handle_http_connection(
    mut stream: TcpStream,
    world: Arc<Mutex<TacticalWorld>>,
    tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; 65536];
    let mut n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    if let Some(hdr_end) = buf[..n].windows(4).position(|w| w == b"\r\n\r\n") {
        let body_start = hdr_end + 4;
        let hdr_str = String::from_utf8_lossy(&buf[..hdr_end]);
        let content_len: usize = hdr_str
            .lines()
            .find_map(|l| {
                let lower = l.to_ascii_lowercase();
                if lower.starts_with("content-length:") {
                    l.split(':').nth(1)?.trim().parse().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let total_needed = body_start + content_len;
        if total_needed > buf.len() && total_needed <= 4 * 1024 * 1024 {
            buf.resize(total_needed, 0);
        }
        while n < total_needed && n < buf.len() {
            let r = stream.read(&mut buf[n..total_needed]).await?;
            if r == 0 {
                break;
            }
            n += r;
        }
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        let header = format!(
            "HTTP/1.1 204 No Content\r\n{CORS_HEADERS}Content-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(header.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && (path == "/" || path == "/index.html") {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n{CORS_HEADERS}Content-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
            INDEX_HTML.len()
        );
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(INDEX_HTML.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path == "/api/state" {
        let body = {
            let guard = world.lock().unwrap();
            let mut entities: Vec<TacticalEntity> = guard.entities.values().cloned().collect();
            entities.sort_by(|a, b| a.uid.cmp(&b.uid));
            let snap = WorldSnapshot {
                origin: guard.origin,
                terrain_name: guard.terrain_name.clone(),
                sim_enabled: guard.sim_enabled,
                sim_time_sec: guard.sim_time_sec,
                aar: guard.aar.summary(),
                entities,
                threat_domes: guard.threat_domes.clone(),
                tactical_zones: guard.tactical_zones.clone(),
                operators: guard.collab.sorted_peers(),
                table_pose: guard.collab.table_pose,
            };
            serde_json::to_string(&snap)?
        };
        let header = json_ok_header(body.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path.starts_with("/api/aar") {
        let target_t: Option<f64> = path
            .split("t=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .and_then(|s| s.parse::<f64>().ok());
        let body = {
            let guard = world.lock().unwrap();
            let summary = guard.aar.summary();
            let t_query = target_t.unwrap_or(summary.max_time_sec);
            let frame = guard.aar.sample_at(t_query);
            let (domes, zones) = frame
                .as_ref()
                .map(|f| guard.evaluate_overlays_for_entities(&f.entities))
                .unwrap_or_default();
            serde_json::json!({
                "summary": summary,
                "frame": frame,
                "threat_domes": domes,
                "tactical_zones": zones
            })
            .to_string()
        };
        let header = json_ok_header(body.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path.starts_with("/api/hud") {
        let obs = path
            .split("observer=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .unwrap_or("GND-SPEAR-ALPHA");
        let body = {
            let guard = world.lock().unwrap();
            let cues = guard.compute_visor_hud_cues(obs);
            serde_json::json!({
                "observer_uid": obs,
                "cues": cues.into_iter().map(|(uid, callsign, affiliation, cue)| serde_json::json!({
                    "uid": uid,
                    "callsign": callsign,
                    "affiliation": affiliation,
                    "bearing_deg": cue.bearing_deg,
                    "slant_range_m": cue.slant_range_m,
                    "elevation_deg": cue.elevation_deg
                })).collect::<Vec<_>>()
            })
            .to_string()
        };
        let header = json_ok_header(body.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path == "/api/terrain" {
        let body = {
            let guard = world.lock().unwrap();
            let res = 65usize;
            let grid = guard.terrain.export_grid(guard.origin.extent_m, res);
            serde_json::json!({
                "name": guard.terrain_name,
                "resolution": res,
                "extent_m": guard.origin.extent_m,
                "heights_m": grid
            })
            .to_string()
        };
        let header = json_ok_header(body.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/terrain" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(t_req) = serde_json::from_str::<TerrainSwitchRequest>(body) {
            let resp_json = {
                let mut guard = world.lock().unwrap();
                let extent = t_req.extent_m.unwrap_or(guard.origin.extent_m);
                if let (Some(w), Some(h), Some(heights)) =
                    (t_req.width, t_req.height, t_req.heights_m)
                {
                    let name = t_req.preset.unwrap_or_else(|| "custom_dem".to_string());
                    if let Ok(dem) = RasterDemProvider::new(&name, w, h, extent, heights) {
                        guard.set_terrain_provider(&name, Box::new(dem));
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
                let res = 65usize;
                let grid = guard.terrain.export_grid(guard.origin.extent_m, res);
                serde_json::json!({
                    "name": guard.terrain_name,
                    "resolution": res,
                    "extent_m": guard.origin.extent_m,
                    "heights_m": grid
                })
                .to_string()
            };
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/terrain/geotiff" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(gt_req) = serde_json::from_str::<GeoTiffUploadRequest>(body) {
            let name = gt_req.name.unwrap_or_else(|| "GEOTIFF-DSM".to_string());
            let extent = {
                let guard = world.lock().unwrap();
                guard.origin.extent_m
            };
            match ingest_geotiff_dsm(&name, extent, 65, &gt_req.bytes) {
                Ok(parsed) => {
                    let resp_json = {
                        let mut guard = world.lock().unwrap();
                        if gt_req.auto_recenter.unwrap_or(true) {
                            if let Some((clat, clon)) = parsed.center_lat_lon {
                                guard.recenter_origin(GeoOrigin {
                                    lat: clat,
                                    lon: clon,
                                    alt_m: parsed.min_asl_m.round(),
                                    extent_m: extent,
                                });
                            } else {
                                guard.origin.alt_m = parsed.min_asl_m.round();
                            }
                        }
                        guard.set_terrain_provider(&name, Box::new(parsed.dem));
                        let grid = guard.terrain.export_grid(guard.origin.extent_m, 65);
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
                        .to_string()
                    };
                    let header = json_ok_header(resp_json.len());
                    stream.write_all(header.as_bytes()).await?;
                    stream.write_all(resp_json.as_bytes()).await?;
                    return Ok(());
                }
                Err(err) => {
                    let msg = serde_json::json!({ "error": err }).to_string();
                    let header = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n",
                        msg.len()
                    );
                    stream.write_all(header.as_bytes()).await?;
                    stream.write_all(msg.as_bytes()).await?;
                    return Ok(());
                }
            }
        }
    }

    if method == "GET" && path == "/api/stream" {
        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n{CORS_HEADERS}\r\n");
        stream.write_all(header.as_bytes()).await?;
        let mut rx = tx.subscribe();
        while let Ok(json) = rx.recv().await {
            let frame = format!("data: {json}\n\n");
            if stream.write_all(frame.as_bytes()).await.is_err() {
                break;
            }
        }
        return Ok(());
    }

    if method == "POST" && path == "/api/cot" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        let res = {
            let mut guard = world.lock().unwrap();
            let r = guard.ingest_cot_xml(body, now_unix_ms());
            if r.is_ok() {
                save_persistent_pins(&guard);
            }
            r
        };
        match res {
            Ok(ent) => {
                let json = serde_json::to_string(&ent)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
            }
            Err(err) => {
                let msg = serde_json::json!({ "error": err }).to_string();
                let header = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n",
                    msg.len()
                );
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(msg.as_bytes()).await?;
            }
        }
        return Ok(());
    }

    if method == "POST" && path == "/api/spi" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(spi_req) = serde_json::from_str::<SpiSlewRequest>(body) {
            let sensor_uid = spi_req
                .sensor_uid
                .unwrap_or_else(|| "UAV-FALCON-01".to_string());
            if spi_req.clear.unwrap_or(false) {
                {
                    let mut guard = world.lock().unwrap();
                    guard.clear_spi_lock(&sensor_uid);
                }
                let json = serde_json::json!({ "status": "cleared", "sensor_uid": sensor_uid })
                    .to_string();
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }

            let east_m = spi_req.east_m.unwrap_or(0.0);
            let north_m = spi_req.north_m.unwrap_or(0.0);
            let res = {
                let mut guard = world.lock().unwrap();
                guard.slew_sensor_to_spi(&sensor_uid, east_m, north_m, now_unix_ms())
            };
            if let Ok(entity) = res {
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/viewshed" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(vs_req) = serde_json::from_str::<ViewshedRequest>(body) {
            let vs = {
                let guard = world.lock().unwrap();
                compute_viewshed_with_provider(
                    guard.terrain.as_ref(),
                    vs_req.east_m,
                    vs_req.north_m,
                    vs_req.height_offset_m.unwrap_or(18.0),
                    vs_req.radius_m.unwrap_or(680.0),
                    72,
                    28,
                )
            };
            let json = serde_json::to_string(&vs)?;
            let header = json_ok_header(json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/los" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(los_req) = serde_json::from_str::<LinkLosRequest>(body) {
            let res = {
                let guard = world.lock().unwrap();
                let from_enu = los_req.from_enu.or_else(|| {
                    los_req
                        .from_uid
                        .as_ref()
                        .and_then(|u| guard.entities.get(u))
                        .map(|e| (e.east_m, e.up_m + 3.0, e.north_m))
                });
                let to_enu = los_req.to_enu.or_else(|| {
                    los_req
                        .to_uid
                        .as_ref()
                        .and_then(|u| guard.entities.get(u))
                        .map(|e| (e.east_m, e.up_m + 3.0, e.north_m))
                });
                match (from_enu, to_enu) {
                    (Some(a), Some(b)) => Some(compute_link_los(guard.terrain.as_ref(), a, b, 48)),
                    _ => None,
                }
            };
            if let Some(los_result) = res {
                let json = serde_json::to_string(&los_result)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/elevation-profile" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(prof_req) = serde_json::from_str::<ElevationProfileRequest>(body) {
            let prof = {
                let guard = world.lock().unwrap();
                guard.compute_elevation_profile(&prof_req.waypoints_en)
            };
            let json = serde_json::to_string(&prof)?;
            let header = json_ok_header(json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/mission/import" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(m_req) = serde_json::from_str::<MissionImportRequest>(body) {
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
            match parsed {
                Ok((name, raw_wps)) => {
                    let resp_json = {
                        let mut guard = world.lock().unwrap();
                        if m_req.auto_recenter.unwrap_or(true) && !raw_wps.is_empty() {
                            let avg_lat =
                                raw_wps.iter().map(|w| w.lat).sum::<f64>() / raw_wps.len() as f64;
                            let avg_lon =
                                raw_wps.iter().map(|w| w.lon).sum::<f64>() / raw_wps.len() as f64;
                            let ext = guard.origin.extent_m;
                            let alt = guard.origin.alt_m;
                            guard.recenter_origin(GeoOrigin {
                                lat: avg_lat,
                                lon: avg_lon,
                                alt_m: alt,
                                extent_m: ext,
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
                        let wps_en: Vec<(f64, f64)> = corridor
                            .waypoints
                            .iter()
                            .map(|w| (w.east_m, w.north_m))
                            .collect();
                        let profile = guard.compute_elevation_profile(&wps_en);
                        serde_json::json!({
                            "origin": guard.origin,
                            "corridor": corridor,
                            "profile": profile,
                        })
                        .to_string()
                    };
                    let header = json_ok_header(resp_json.len());
                    stream.write_all(header.as_bytes()).await?;
                    stream.write_all(resp_json.as_bytes()).await?;
                    return Ok(());
                }
                Err(err) => {
                    let msg = serde_json::json!({ "error": err }).to_string();
                    let header = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n",
                        msg.len()
                    );
                    stream.write_all(header.as_bytes()).await?;
                    stream.write_all(msg.as_bytes()).await?;
                    return Ok(());
                }
            }
        }
    }

    if method == "POST" && path == "/api/mission/export" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(ex_req) = serde_json::from_str::<MissionExportRequest>(body) {
            let route_uid = ex_req
                .route_uid
                .unwrap_or_else(|| "RTE-XR-TAK-01".to_string());
            let route_cs = ex_req
                .route_callsign
                .unwrap_or_else(|| "XR-CORRIDOR-01".to_string());
            let (qgc_plan_json, atak_bmr_xml) = {
                let guard = world.lock().unwrap();
                let qgc = export_qgc_plan_json(
                    &guard.origin,
                    guard.terrain.as_ref(),
                    &ex_req.waypoints_en_agl,
                );
                let bmr = export_atak_route_cot_xml(
                    &route_uid,
                    &route_cs,
                    &guard.origin,
                    guard.terrain.as_ref(),
                    &ex_req.waypoints_en_agl,
                );
                (qgc, bmr)
            };
            if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = sock
                    .send_to(atak_bmr_xml.as_bytes(), "127.0.0.1:4243")
                    .await;
            }
            let resp_json = serde_json::json!({
                "route_uid": route_uid,
                "route_callsign": route_cs,
                "waypoint_count": ex_req.waypoints_en_agl.len(),
                "qgc_plan": serde_json::from_str::<serde_json::Value>(&qgc_plan_json).unwrap_or(serde_json::Value::Null),
                "qgc_plan_json": qgc_plan_json,
                "atak_bmr_cot_xml": atak_bmr_xml,
            })
            .to_string();
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/tiles/resolve" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(t_req) = serde_json::from_str::<TileResolveRequest>(body) {
            let (lat, lon) = {
                let guard = world.lock().unwrap();
                (
                    t_req.lat.unwrap_or(guard.origin.lat),
                    t_req.lon.unwrap_or(guard.origin.lon),
                )
            };
            let provider = ImageryProvider::from_id(t_req.provider.as_deref().unwrap_or("esri"));
            let zoom = t_req.zoom.unwrap_or(15);
            let spec = resolve_tile_with_fallback(provider, lat, lon, zoom);
            let resp_json = serde_json::to_string(&spec)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/mission/corridor-polygon" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(c_req) = serde_json::from_str::<CorridorPolygonRequest>(body) {
            let origin = {
                let guard = world.lock().unwrap();
                guard.origin
            };
            let res = evaluate_swath_corridor(
                c_req
                    .name
                    .unwrap_or_else(|| "XR-SWATH-CORRIDOR".to_string()),
                &origin,
                &c_req.waypoints_en,
                c_req.rdp_epsilon_m.unwrap_or(8.0),
                c_req.agl_m.unwrap_or(140.0),
                c_req.hfov_deg,
                c_req.focal_px,
                c_req.sensor_width_px,
            );
            let resp_json = serde_json::to_string(&res)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/mapset/plan" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(mp_req) = serde_json::from_str::<MapsetPlanRequest>(body) {
            let origin = {
                let guard = world.lock().unwrap();
                guard.origin
            };
            // Default to the current tactical sand table bounding box if explicit bounds are omitted
            let (sw_lat, sw_lon, _) = origin.enu_to_lla(-origin.extent_m, 0.0, -origin.extent_m);
            let (ne_lat, ne_lon, _) = origin.enu_to_lla(origin.extent_m, 0.0, origin.extent_m);
            let south = mp_req.south.unwrap_or(sw_lat);
            let west = mp_req.west.unwrap_or(sw_lon);
            let north = mp_req.north.unwrap_or(ne_lat);
            let east = mp_req.east.unwrap_or(ne_lon);
            let provider = ImageryProvider::from_id(mp_req.provider.as_deref().unwrap_or("bing"));
            let min_zoom = mp_req.min_zoom.unwrap_or(12);
            let max_zoom = mp_req.max_zoom.unwrap_or(16);
            let name = mp_req.name.unwrap_or_default();
            let manifest = build_mapset_manifest(
                &name, provider, south, west, north, east, min_zoom, max_zoom,
            );
            let resp_json = serde_json::to_string(&manifest)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/rf/link-budget" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(rf_req) = serde_json::from_str::<RfLinkBudgetRequest>(body) {
            let tx = rf_req.tx_enu.unwrap_or((-600.0, -400.0, 150.0));
            let rx = rf_req.rx_enu.unwrap_or((800.0, 600.0, 180.0));
            let band = match rf_req.band.as_deref().unwrap_or("ism2400") {
                "uhf900" | "900" => RfBand::UHF900,
                "shf5800" | "5800" | "5.8" => RfBand::SHF5800,
                _ => RfBand::ISM2400,
            };
            let steps = rf_req.steps.unwrap_or(32);
            let result = {
                let guard = world.lock().unwrap();
                evaluate_rf_link(tx, rx, band, steps, guard.terrain.as_ref())
            };
            let resp_json = serde_json::to_string(&result)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/tracks/predict" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        let p_req: TrackPredictRequest =
            serde_json::from_str(body).unwrap_or_else(|_| TrackPredictRequest {
                horizons_sec: Some(vec![15.0, 30.0, 60.0]),
                stale_threshold_sec: Some(5.0),
            });
        let horizons = p_req.horizons_sec.unwrap_or_else(|| vec![15.0, 30.0, 60.0]);
        let stale_thresh = p_req.stale_threshold_sec.unwrap_or(5.0);
        let predictions: Vec<TrackPrediction> = {
            let guard = world.lock().unwrap();
            guard
                .entities
                .values()
                .map(|e| {
                    let is_stale = e.speed_mps <= stale_thresh;
                    predict_entity_trajectory(
                        &e.uid,
                        &e.callsign,
                        (e.east_m, e.north_m, e.up_m),
                        e.speed_mps,
                        e.course_deg,
                        is_stale,
                        5.0,
                        &horizons,
                    )
                })
                .collect()
        };
        let resp_json = serde_json::to_string(&predictions)?;
        let header = json_ok_header(resp_json.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(resp_json.as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/airspace/deconflict" {
        let deconflict_res = {
            let guard = world.lock().unwrap();
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
            evaluate_theater_deconfliction(&list)
        };
        let resp_json = serde_json::to_string(&deconflict_res)?;
        let header = json_ok_header(resp_json.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(resp_json.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path == "/api/export/opord" {
        let bundle = {
            let guard = world.lock().unwrap();
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
            let zone_count = guard.tactical_zones.len();
            let dome_count = guard.threat_domes.len();
            build_opord_bundle(
                &format!("XRTAK-{}", guard.terrain_name.to_uppercase()),
                origin_lla,
                &units,
                zone_count,
                dome_count,
            )
        };
        let resp_json = serde_json::to_string(&bundle)?;
        let header = json_ok_header(resp_json.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(resp_json.as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/ew/jamming" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(ew_req) = serde_json::from_str::<ew_dto::EwJammingRequest>(body) {
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
                let dx_rj = ew_req.jammer.east_m;
                let dy_rj = ew_req.jammer.north_m;
                let dz_rj = ew_req.jammer.up_m;
                let d_rj = (dx_rj.powi(2) + dy_rj.powi(2) + dz_rj.powi(2)).sqrt();
                xrtak_core::compute_burn_through_range(
                    "GROUND-RADAR",
                    r.radar_power_w,
                    r.radar_gain_dbi,
                    r.target_rcs_m2,
                    &ew_req.jammer.callsign,
                    ew_req.jammer.transmitter_power_w,
                    ew_req.jammer.antenna_gain_dbi,
                    d_rj,
                    r.current_target_range_m,
                )
            });
            let resp = ew_dto::EwJammingResponse {
                assessment,
                burn_through,
            };
            let resp_json = serde_json::to_string(&resp)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
        let err = r#"{"error":"invalid_ew_jamming_payload"}"#;
        stream.write_all(format!("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n{}", err.len(), err).as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/medevac/nine-line" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(m_req) = serde_json::from_str::<medevac_dto::MedevacNineLineRequest>(body) {
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
                let guard = world.lock().unwrap();
                let elev: &dyn xrtak_core::ElevationProvider = guard.terrain.as_ref();
                let suit =
                    xrtak_core::evaluate_hlz_suitability(e, n, radius, elev, &guard.threat_domes);
                let (lat, lon, _) = guard.origin.enu_to_lla(e, n, u);
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
                line6_security: m_req.security.unwrap_or_else(|| "N - No enemy troops in area".to_string()),
                line7_hlz_marking: m_req.marking.unwrap_or_else(|| "Panels / Smoke Green".to_string()),
                line8_patient_nationality: m_req.nationality.unwrap_or_else(|| "Friendly Military".to_string()),
                line9_nbc_or_terrain: m_req.terrain_remarks.unwrap_or_else(|| "Valley clearing".to_string()),
                lat,
                lon,
                east_m: e,
                north_m: n,
                up_m: u,
            };
            let formatted = xrtak_core::format_nato_nine_line_text(&report);
            let resp = medevac_dto::MedevacNineLineResponse {
                formatted_nine_line: formatted,
                report,
                hlz_suitability,
            };
            let resp_json = serde_json::to_string(&resp)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
        let err = r#"{"error":"invalid_medevac_request_payload"}"#;
        stream.write_all(format!("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n{}", err.len(), err).as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/fires/ballistics" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(b_req) = serde_json::from_str::<fires_dto::BallisticsRequest>(body) {
            let munition = match b_req.munition.as_deref() {
                Some("mortar") | Some("mortar120") => xrtak_core::MunitionType::Mortar120mm,
                Some("loitering") | Some("drone") => xrtak_core::MunitionType::LoiteringMunition,
                Some("rocket") | Some("mlrs") => xrtak_core::MunitionType::TacticalRocket,
                _ => xrtak_core::MunitionType::Artillery155mm,
            };
            let samples = b_req.samples.unwrap_or(32);
            let solution = {
                let guard = world.lock().unwrap();
                let elev: &dyn xrtak_core::ElevationProvider = guard.terrain.as_ref();
                xrtak_core::compute_ballistic_trajectory(
                    b_req.origin_enu,
                    b_req.target_enu,
                    munition,
                    b_req.elevation_angle_deg,
                    elev,
                    samples,
                )
            };
            let resp = fires_dto::BallisticsResponse { solution };
            let resp_json = serde_json::to_string(&resp)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
        let err = r#"{"error":"invalid_ballistics_request_payload"}"#;
        stream.write_all(format!("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n{}", err.len(), err).as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/uav/endurance" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(bat_req) = serde_json::from_str::<battery_dto::UavEnduranceRequest>(body) {
            let wind = bat_req.wind.unwrap_or_default();
            let steps = bat_req.radial_steps.unwrap_or(32);
            let solution = xrtak_core::compute_uav_endurance(&bat_req.state, &wind, steps);
            let resp = battery_dto::UavEnduranceResponse { solution };
            let resp_json = serde_json::to_string(&resp)?;
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
        let err = r#"{"error":"invalid_uav_endurance_payload"}"#;
        stream.write_all(format!("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n{}", err.len(), err).as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/spawn" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(spawn) = serde_json::from_str::<SpawnRequest>(body) {
            let (xml, ent) = {
                let mut guard = world.lock().unwrap();
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
                    cot_type = spawn.cot_type,
                    callsign = spawn.callsign
                );
                let r = guard.ingest_cot_xml(&xml, now_unix_ms());
                if r.is_ok() {
                    save_persistent_pins(&guard);
                }
                (xml, r)
            };
            if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = sock.send_to(xml.as_bytes(), "127.0.0.1:4243").await;
            }
            if let Ok(entity) = ent {
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/pin/move" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(mv) = serde_json::from_str::<PinMoveRequest>(body) {
            let updated = {
                let mut guard = world.lock().unwrap();
                let ground_h = guard.sample_relief_m(mv.east_m, mv.north_m);
                let up_m = ground_h + mv.alt_offset_m.unwrap_or(0.0);
                let (lat, lon, hae) = guard.origin.enu_to_lla(mv.east_m, up_m, mv.north_m);
                let mut out_ent: Option<(String, TacticalEntity)> = None;
                if let Some(ent) = guard.entities.get_mut(&mv.uid) {
                    ent.east_m = mv.east_m;
                    ent.north_m = mv.north_m;
                    ent.up_m = up_m;
                    ent.lat = lat;
                    ent.lon = lon;
                    ent.hae_m = hae;
                    if let Some(cs) = mv.callsign {
                        if !cs.trim().is_empty() {
                            ent.callsign = cs;
                        }
                    }
                    if let Some(rmk) = mv.remarks {
                        ent.remarks = rmk;
                    }
                    ent.updated_unix_ms = now_unix_ms();
                    let cloned = ent.clone();
                    let xml = entity_to_cot_xml(&cloned);
                    out_ent = Some((xml, cloned));
                }
                if out_ent.is_some() {
                    guard.refresh_threat_domes_and_zones();
                    save_persistent_pins(&guard);
                }
                out_ent
            };
            if let Some((xml, entity)) = updated {
                if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
                    let _ = sock.send_to(xml.as_bytes(), "127.0.0.1:4243").await;
                }
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/pin/delete" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(del) = serde_json::from_str::<PinDeleteRequest>(body) {
            let removed = {
                let mut guard = world.lock().unwrap();
                let r = guard.entities.remove(&del.uid);
                guard.refresh_threat_domes_and_zones();
                save_persistent_pins(&guard);
                r.is_some()
            };
            let json = serde_json::json!({ "deleted": removed, "uid": del.uid }).to_string();
            let header = json_ok_header(json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "GET" && path == "/api/export/geojson" {
        let body = {
            let guard = world.lock().unwrap();
            let mut ents: Vec<&TacticalEntity> = guard.entities.values().collect();
            ents.sort_by(|a, b| a.uid.cmp(&b.uid));
            let features: Vec<serde_json::Value> = ents
                .into_iter()
                .map(|e| {
                    serde_json::json!({
                        "type": "Feature",
                        "id": e.uid,
                        "geometry": {
                            "type": "Point",
                            "coordinates": [e.lon, e.lat, e.hae_m]
                        },
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
            serde_json::json!({
                "type": "FeatureCollection",
                "origin": guard.origin,
                "features": features
            })
            .to_string()
        };
        let header = json_ok_header(body.len());
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path == "/api/export/cot" {
        let body = {
            let guard = world.lock().unwrap();
            let mut ents: Vec<&TacticalEntity> = guard.entities.values().collect();
            ents.sort_by(|a, b| a.uid.cmp(&b.uid));
            let events: Vec<String> = ents.into_iter().map(entity_to_cot_xml).collect();
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<events origin_lat=\"{:.6}\" origin_lon=\"{:.6}\">\n  {}\n</events>\n",
                guard.origin.lat,
                guard.origin.lon,
                events.join("\n  ")
            )
        };
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/xml; charset=utf-8\r\n{CORS_HEADERS}Content-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/api/pointer" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(ptr_req) = serde_json::from_str::<LaserPointerRequest>(body) {
            let (xml, res) = {
                let mut guard = world.lock().unwrap();
                let up_m = guard.sample_relief_m(ptr_req.east_m, ptr_req.north_m);
                let (lat, lon, hae) = guard.origin.enu_to_lla(ptr_req.east_m, up_m, ptr_req.north_m);
                let cs = ptr_req.callsign.unwrap_or_else(|| "QUEST3-OP".to_string());
                let xml = build_laser_pointer_cot_xml(&cs, lat, lon, hae);
                let res = guard.ingest_cot_xml(&xml, now_unix_ms());
                (xml, res)
            };
            if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = sock.send_to(xml.as_bytes(), "127.0.0.1:4243").await;
            }
            if let Ok(entity) = res {
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/collab/sync" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(c_req) = serde_json::from_str::<CollabSyncRequest>(body) {
            let resp_json = {
                let mut guard = world.lock().unwrap();
                let now_ms = now_unix_ms();
                let peer = OperatorPeer {
                    operator_id: c_req.operator_id,
                    callsign: c_req.callsign,
                    device_mode: c_req.device_mode.unwrap_or_else(|| "DESKTOP-C2".to_string()),
                    head_enu: c_req.head_enu.unwrap_or((0.0, 480.0, -420.0)),
                    laser_target_enu: c_req.laser_target_enu,
                    ptt_active: c_req.ptt_active.unwrap_or(false),
                    color_hex: c_req.color_hex.unwrap_or_else(|| "#00ff88".to_string()),
                    updated_unix_ms: now_ms,
                };
                guard
                    .collab
                    .upsert_peer(peer, c_req.shared_scale, c_req.shared_yaw_deg);
                serde_json::json!({
                    "operators": guard.collab.sorted_peers(),
                    "table_pose": guard.collab.table_pose,
                })
                .to_string()
            };
            let header = json_ok_header(resp_json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp_json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/sapient" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(report) = serde_json::from_str::<SapientDetectionReport>(body) {
            let res = {
                let mut guard = world.lock().unwrap();
                guard.ingest_sapient_detection(&report, now_unix_ms())
            };
            if let Ok((xml, entity)) = res {
                if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
                    let _ = sock.send_to(xml.as_bytes(), "127.0.0.1:4243").await;
                }
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/tak-config" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(cfg) = serde_json::from_str::<TakConfigRequest>(body) {
            if let Some(addr) = cfg.tak_tcp_addr.clone() {
                if !addr.trim().is_empty() {
                    let world_tcp = Arc::clone(&world);
                    tokio::spawn(async move {
                        if let Ok(mut tcp) = TcpStream::connect(addr.trim()).await {
                            println!("[XR-TAK] Connected to Live TAK Server TCP stream at {addr}");
                            let mut buf = vec![0u8; 65535];
                            while let Ok(n) = tcp.read(&mut buf).await {
                                if n == 0 {
                                    break;
                                }
                                let chunk = String::from_utf8_lossy(&buf[..n]);
                                for part in chunk.split("</event>") {
                                    if let Some(start) = part.find("<event") {
                                        let xml = format!("{}</event>", &part[start..]);
                                        let mut g = world_tcp.lock().unwrap();
                                        let _ = g.ingest_cot_xml(&xml, now_unix_ms());
                                    }
                                }
                            }
                        }
                    });
                }
            }
            let resp = {
                let mut guard = world.lock().unwrap();
                if let Some(sim) = cfg.sim_enabled {
                    guard.set_sim_enabled(sim);
                }
                if let Some(auto_rc) = cfg.auto_recenter {
                    guard.auto_recenter_on_next_live_cot = auto_rc;
                }
                if let (Some(lat), Some(lon)) = (cfg.origin_lat, cfg.origin_lon) {
                    let alt_m = cfg.origin_alt_m.unwrap_or(guard.origin.alt_m);
                    let ext = guard.origin.extent_m;
                    guard.recenter_origin(GeoOrigin { lat, lon, alt_m, extent_m: ext });
                }
                save_persistent_pins(&guard);
                serde_json::json!({
                    "origin": guard.origin,
                    "sim_enabled": guard.sim_enabled,
                    "auto_recenter": guard.auto_recenter_on_next_live_cot
                })
                .to_string()
            };
            let header = json_ok_header(resp.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp.as_bytes()).await?;
            return Ok(());
        }
    }

    let not_found =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
    stream.write_all(not_found.as_bytes()).await?;
    Ok(())
}
