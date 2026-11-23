use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{broadcast, Semaphore};
use xrtak_core::{
    build_laser_pointer_cot_xml, escape_cot_attr, evaluate_rf_link, evaluate_theater_deconfliction,
    predict_entity_trajectory_struct, AarSummary, GeoOrigin, OperatorPeer, OpordUnitSummary,
    RfBand, SapientDetectionReport, SharedTablePose, TacticalEntity, TacticalWorld, TacticalZone,
    ThreatDome, TrackPredictInput, TrackPredictRequest, TrackPrediction,
};

mod battery_dto;
mod ew_dto;
mod fires_dto;
mod http;
mod medevac_dto;
mod router;

use http::{HttpResponse, CORS_HEADERS};

const INDEX_HTML: &str = include_str!("../../../index.html");

fn json_ok_header(len: usize) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{CORS_HEADERS}Content-Length: {len}\r\nConnection: close\r\n\r\n"
    )
}

fn lock_world(world: &Mutex<TacticalWorld>) -> MutexGuard<'_, TacticalWorld> {
    world.lock().unwrap_or_else(PoisonError::into_inner)
}

async fn send_err_400(stream: &mut TcpStream, err: &str) -> Result<(), http::DynError> {
    http::write_response(stream, &HttpResponse::bad_request(err)).await
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut res = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        res |= x ^ y;
    }
    res == 0
}

fn normalize_host(h: &str) -> &str {
    let s = h.trim();
    let s = if let Some(idx) = s.find("://") {
        &s[idx + 3..]
    } else {
        s
    };
    s.split('/').next().unwrap_or(s)
}

fn is_state_changing(method: &str, path: &str) -> bool {
    method == "POST"
        && matches!(
            path,
            "/api/cot"
                | "/api/spawn"
                | "/api/pin/move"
                | "/api/pin/delete"
                | "/api/sapient"
                | "/api/pointer"
                | "/api/spi"
                | "/api/collab/sync"
                | "/api/terrain"
                | "/api/terrain/geotiff"
                | "/api/mission/import"
                | "/api/tak-config"
        )
}

fn check_request_auth(req: &http::HttpRequest) -> Option<(u16, &'static str)> {
    let origin_hdr = req.header("origin");
    let host_hdr = req.header("host");
    let token_hdr = req.header("x-xrtak-token");
    if let Some(origin) = origin_hdr {
        let orig = normalize_host(origin);
        let host = normalize_host(host_hdr.unwrap_or(""));
        if !orig.eq_ignore_ascii_case(host) {
            return Some((403, r#"{"error":"forbidden_cross_origin"}"#));
        }
    }
    if let Ok(expected) = std::env::var("XRTAK_TOKEN") {
        match token_hdr {
            Some(provided) => {
                if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
                    return Some((401, r#"{"error":"unauthorized"}"#));
                }
            }
            None => {
                return Some((401, r#"{"error":"unauthorized"}"#));
            }
        }
    }
    None
}

fn feedback_addr() -> String {
    std::env::var("XRTAK_FEEDBACK_ADDR").unwrap_or_else(|_| "127.0.0.1:4243".to_string())
}

struct GatewayState {
    world: Arc<Mutex<TacticalWorld>>,
    tx: broadcast::Sender<String>,
    tak_client: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    feedback_sock: Option<UdpSocket>,
    feedback_addr: String,
}

impl GatewayState {
    async fn send_feedback(&self, payload: &[u8]) {
        if let Some(sock) = &self.feedback_sock {
            let _ = sock.send_to(payload, &self.feedback_addr).await;
        }
    }
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

impl WorldSnapshot {
    fn from_world(world: &TacticalWorld) -> Self {
        let mut entities: Vec<TacticalEntity> = world.entities.values().cloned().collect();
        entities.sort_by(|a, b| a.uid.cmp(&b.uid));
        Self {
            origin: world.origin,
            terrain_name: world.terrain_name.clone(),
            sim_enabled: world.sim_enabled,
            sim_time_sec: world.sim_time_sec,
            aar: world.aar.summary(),
            entities,
            threat_domes: world.threat_domes.clone(),
            tactical_zones: world.tactical_zones.clone(),
            operators: world.collab.sorted_peers(),
            table_pose: world.collab.table_pose,
        }
    }
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
    #[serde(deserialize_with = "deserialize_bounded_waypoints")]
    waypoints_en: Vec<(f64, f64)>,
}

fn deserialize_bounded_waypoints<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<(f64, f64)>, D::Error> {
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<(f64, f64)>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded waypoint array")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let cap = xrtak_core::geo::MAX_ELEVATION_PROFILE_WAYPOINTS;
            let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(cap));
            while let Some(pt) = seq.next_element()? {
                if out.len() >= cap {
                    return Err(serde::de::Error::custom("too many waypoints"));
                }
                out.push(pt);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_seq(Visitor)
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
    rgb_sample: Option<Vec<u8>>,
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

#[allow(dead_code)]
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

static OPERATOR_PINS: std::sync::LazyLock<Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

fn snapshot_persistent_pins_json(world: &TacticalWorld) -> Option<String> {
    let op_pins = OPERATOR_PINS.lock().unwrap_or_else(PoisonError::into_inner);
    let mut pins: Vec<&TacticalEntity> = world
        .entities
        .values()
        .filter(|e| op_pins.contains(&e.uid))
        .collect();
    pins.sort_by(|a, b| a.uid.cmp(&b.uid));
    serde_json::to_string_pretty(&pins).ok()
}

async fn flush_persistent_pins(pins_json: Option<String>) {
    if let Some(json) = pins_json {
        let _ = tokio::task::spawn_blocking(move || {
            let _ = std::fs::create_dir_all("data");
            let _ = std::fs::write(PINS_STORE_PATH, json);
        })
        .await;
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
                OPERATOR_PINS.lock().unwrap().insert(p.uid.clone());
                world.entities.insert(p.uid.clone(), p);
            }
            world.refresh_threat_domes_and_zones();
        }
    }
}

fn entity_to_cot_xml(ent: &TacticalEntity) -> String {
    format!(
        r#"<event version="2.0" uid="{uid}" type="{cot_type}" how="h-g-i-g-o"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae:.1}" ce="5.0" le="5.0"/><detail><contact callsign="{callsign}"/><track course="{course:.1}" speed="{speed:.1}"/><remarks>{remarks}</remarks></detail></event>"#,
        uid = escape_cot_attr(&ent.uid),
        cot_type = escape_cot_attr(&ent.cot_type),
        lat = ent.lat,
        lon = ent.lon,
        hae = ent.hae_m,
        callsign = escape_cot_attr(&ent.callsign),
        course = ent.course_deg,
        speed = ent.speed_mps,
        remarks = escape_cot_attr(&ent.remarks)
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
        std::env::var("XRTAK_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:8090".to_string());
    let udp_addr_str =
        std::env::var("XRTAK_UDP_ADDR").unwrap_or_else(|_| "127.0.0.1:4242".to_string());
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
                            let mut guard = lock_world(&world_udp);
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
                    let mut guard = lock_world(&world_sim);
                    let now_ms = now_unix_ms();
                    guard.step_simulation(0.05, now_ms);
                    guard.collab.prune_stale(now_ms, 15_000);
                    let snap = WorldSnapshot::from_world(&guard);
                    serde_json::to_string(&snap).unwrap_or_default()
                };
                let _ = tx_sim.send(payload);
            }
        });
    }

    // 3. HTTP + SSE + CoT/SPI/Viewshed Server on :8090 (with automatic fallback to :8091)
    let listener = match TcpListener::bind(http_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[XR-TAK] Could not bind HTTP {http_addr}: {e}");
            let fallback_addr = SocketAddr::new(http_addr.ip(), 8091);
            TcpListener::bind(fallback_addr).await?
        }
    };
    let bound_port = listener.local_addr()?.port();
    println!("==============================================================");
    println!("  XR-TAK — Rust CoT Core + WebXR 3D Tactical Table");
    println!("  WebXR / Browser UI : http://localhost:{bound_port}");
    println!("  Raw CoT UDP Input  : udp://{udp_cot_addr}");
    println!("==============================================================");

    let fb_addr = feedback_addr();
    let fb_bind = if fb_addr.trim_start().starts_with('[')
        || fb_addr.parse::<SocketAddr>().is_ok_and(|a| a.is_ipv6())
    {
        "[::]:0"
    } else {
        "0.0.0.0:0"
    };
    let state = Arc::new(GatewayState {
        world,
        tx,
        tak_client: Arc::new(Mutex::new(None::<tokio::task::JoinHandle<()>>)),
        feedback_sock: UdpSocket::bind(fb_bind).await.ok(),
        feedback_addr: fb_addr,
    });
    let conn_limit = Arc::new(Semaphore::new(256));
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[gateway] accept error: {e}");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let permit = match conn_limit.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let state_http = Arc::clone(&state);
        tokio::spawn(async move {
            let _permit = permit;
            let _ = handle_http_connection(stream, state_http).await;
        });
    }
}

async fn handle_http_connection(
    mut stream: TcpStream,
    state: Arc<GatewayState>,
) -> Result<(), http::DynError> {
    let (world, tx, tak_client) = (
        Arc::clone(&state.world),
        state.tx.clone(),
        Arc::clone(&state.tak_client),
    );
    let Some(req) = http::read_request(&mut stream).await? else {
        return Ok(());
    };
    let (method, path, body) = (req.method.as_str(), req.target.as_str(), req.body.as_str());

    if is_state_changing(method, path) {
        if let Some((code, err)) = check_request_auth(&req) {
            return http::write_response(&mut stream, &HttpResponse::auth_error(code, err)).await;
        }
    }

    if let Some(resp) = router::try_dispatch(&req, &state).await? {
        return http::write_response(&mut stream, &resp).await;
    }

    if method == "GET" && path == "/api/stream" {
        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n{CORS_HEADERS}\r\n");
        stream.write_all(header.as_bytes()).await?;
        let mut rx = tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(json) => {
                    let frame = format!("data: {json}\n\n");
                    if stream.write_all(frame.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
        return Ok(());
    }

    if method == "POST" && path == "/api/ew/jamming" {
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
                let guard = lock_world(&world);
                let elev: &dyn xrtak_core::ElevationProvider = guard.terrain.as_ref();
                let suit =
                    xrtak_core::evaluate_hlz_suitability(e, n, radius, elev, &guard.threat_domes);
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
        if let Ok(b_req) = serde_json::from_str::<fires_dto::BallisticsRequest>(body) {
            let munition = match b_req.munition.as_deref() {
                Some("mortar") | Some("mortar120") => xrtak_core::MunitionType::Mortar120mm,
                Some("loitering") | Some("drone") => xrtak_core::MunitionType::LoiteringMunition,
                Some("rocket") | Some("mlrs") => xrtak_core::MunitionType::TacticalRocket,
                _ => xrtak_core::MunitionType::Artillery155mm,
            };
            let samples = b_req.samples.unwrap_or(32);
            let solution = {
                let guard = lock_world(&world);
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
        if let Ok(spawn) = serde_json::from_str::<SpawnRequest>(body) {
            let (xml, ent, pins_json) = {
                let mut guard = lock_world(&world);
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
                        .insert(uid.clone());
                    snapshot_persistent_pins_json(&guard)
                } else {
                    None
                };
                (xml, r, pj)
            };
            flush_persistent_pins(pins_json).await;
            state.send_feedback(xml.as_bytes()).await;
            if let Ok(entity) = ent {
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
        return send_err_400(&mut stream, "invalid_spawn_payload").await;
    }

    if method == "POST" && path == "/api/pin/move" {
        if let Ok(mv) = serde_json::from_str::<PinMoveRequest>(body) {
            let (updated, pins_json) = {
                let mut guard = lock_world(&world);
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
                let pj = if out_ent.is_some() {
                    guard.refresh_threat_domes_and_zones();
                    snapshot_persistent_pins_json(&guard)
                } else {
                    None
                };
                (out_ent, pj)
            };
            flush_persistent_pins(pins_json).await;
            if let Some((xml, entity)) = updated {
                state.send_feedback(xml.as_bytes()).await;
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
        return send_err_400(&mut stream, "invalid_pin_move_payload").await;
    }

    if method == "POST" && path == "/api/pin/delete" {
        if let Ok(del) = serde_json::from_str::<PinDeleteRequest>(body) {
            let (removed, pins_json) = {
                let mut guard = lock_world(&world);
                let r = guard.entities.remove(&del.uid);
                OPERATOR_PINS
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .remove(&del.uid);
                guard.refresh_threat_domes_and_zones();
                (r.is_some(), snapshot_persistent_pins_json(&guard))
            };
            flush_persistent_pins(pins_json).await;
            let json = serde_json::json!({ "deleted": removed, "uid": del.uid }).to_string();
            let header = json_ok_header(json.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(json.as_bytes()).await?;
            return Ok(());
        }
        return send_err_400(&mut stream, "invalid_pin_delete_payload").await;
    }

    if method == "GET" && path == "/api/export/geojson" {
        let body = {
            let guard = lock_world(&world);
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
            let guard = lock_world(&world);
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
        return http::write_response(&mut stream, &HttpResponse::xml(body)).await;
    }

    if method == "POST" && path == "/api/pointer" {
        if let Ok(ptr_req) = serde_json::from_str::<LaserPointerRequest>(body) {
            let (xml, res) = {
                let mut guard = lock_world(&world);
                let up_m = guard.sample_relief_m(ptr_req.east_m, ptr_req.north_m);
                let (lat, lon, hae) =
                    guard
                        .origin
                        .enu_to_lla(ptr_req.east_m, up_m, ptr_req.north_m);
                let cs = ptr_req.callsign.unwrap_or_else(|| "QUEST3-OP".to_string());
                let xml = build_laser_pointer_cot_xml(&cs, lat, lon, hae);
                let res = guard.ingest_cot_xml(&xml, now_unix_ms());
                (xml, res)
            };
            state.send_feedback(xml.as_bytes()).await;
            if let Ok(entity) = res {
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
        return send_err_400(&mut stream, "invalid_pointer_payload").await;
    }

    if method == "POST" && path == "/api/collab/sync" {
        if let Ok(c_req) = serde_json::from_str::<CollabSyncRequest>(body) {
            let resp_json = {
                let mut guard = lock_world(&world);
                let now_ms = now_unix_ms();
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
                    updated_unix_ms: now_ms,
                };
                if !guard.collab.peers.contains_key(&peer.operator_id)
                    && guard.collab.peers.len() >= 64
                {
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
        return send_err_400(&mut stream, "invalid_collab_sync_payload").await;
    }

    if method == "POST" && path == "/api/sapient" {
        if let Ok(report) = serde_json::from_str::<SapientDetectionReport>(body) {
            let res = {
                let mut guard = lock_world(&world);
                guard.ingest_sapient_detection(&report, now_unix_ms())
            };
            if let Ok((xml, entity)) = res {
                state.send_feedback(xml.as_bytes()).await;
                let json = serde_json::to_string(&entity)?;
                let header = json_ok_header(json.len());
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
        return send_err_400(&mut stream, "invalid_sapient_payload").await;
    }

    if method == "POST" && path == "/api/tak-config" {
        if let Ok(cfg) = serde_json::from_str::<TakConfigRequest>(body) {
            if let Some(addr) = cfg.tak_tcp_addr.clone() {
                if !addr.trim().is_empty() {
                    let world_tcp = Arc::clone(&world);
                    let handle = tokio::spawn(async move {
                        if let Ok(mut tcp) = TcpStream::connect(addr.trim()).await {
                            println!("[XR-TAK] Connected to Live TAK Server TCP stream at {addr}");
                            let mut buf = vec![0u8; 65535];
                            let mut carry = String::new();
                            while let Ok(n) = tcp.read(&mut buf).await {
                                if n == 0 {
                                    break;
                                }
                                carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                                while let Some(end_idx) = carry.find("</event>") {
                                    let end = end_idx + "</event>".len();
                                    let segment = carry[..end].to_string();
                                    carry = carry[end..].to_string();
                                    if let Some(start) = segment.find("<event") {
                                        let xml = &segment[start..];
                                        let mut g = lock_world(&world_tcp);
                                        let _ = g.ingest_cot_xml(xml, now_unix_ms());
                                    }
                                }
                            }
                        }
                    });
                    let mut lock = tak_client.lock().unwrap_or_else(PoisonError::into_inner);
                    if let Some(prev) = lock.take() {
                        prev.abort();
                    }
                    *lock = Some(handle);
                }
            }
            let (resp, pins_json) = {
                let mut guard = lock_world(&world);
                if let Some(sim) = cfg.sim_enabled {
                    guard.set_sim_enabled(sim);
                }
                if let Some(auto_rc) = cfg.auto_recenter {
                    guard.auto_recenter_on_next_live_cot = auto_rc;
                }
                if let (Some(lat), Some(lon)) = (cfg.origin_lat, cfg.origin_lon) {
                    let alt_m = cfg.origin_alt_m.unwrap_or(guard.origin.alt_m);
                    let ext = guard.origin.extent_m;
                    guard.recenter_origin(GeoOrigin {
                        lat,
                        lon,
                        alt_m,
                        extent_m: ext,
                    });
                }
                let pj = snapshot_persistent_pins_json(&guard);
                let out = serde_json::json!({
                    "origin": guard.origin,
                    "sim_enabled": guard.sim_enabled,
                    "auto_recenter": guard.auto_recenter_on_next_live_cot
                })
                .to_string();
                (out, pj)
            };
            flush_persistent_pins(pins_json).await;
            let header = json_ok_header(resp.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(resp.as_bytes()).await?;
            return Ok(());
        }
        return send_err_400(&mut stream, "invalid_tak_config_payload").await;
    }

    http::write_response(&mut stream, &HttpResponse::not_found()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_html_matches_root_source() {
        let root_index = include_str!("../../../index.html");
        assert!(!INDEX_HTML.trim().is_empty());
        assert_eq!(INDEX_HTML, root_index);
    }
}
