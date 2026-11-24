use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{broadcast, watch, Semaphore};
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
pub mod tak_client;

const INDEX_HTML: &str = include_str!("../../../index.html");

fn lock_world(world: &Mutex<TacticalWorld>) -> MutexGuard<'_, TacticalWorld> {
    world.lock().unwrap_or_else(PoisonError::into_inner)
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
    tak_status_tx: watch::Sender<tak_client::TakClientStatus>,
    tak_status_rx: watch::Receiver<tak_client::TakClientStatus>,
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
    tls_enabled: Option<bool>,
    ca_cert_pem: Option<String>,
    client_cert_pem: Option<String>,
    client_key_pem: Option<String>,
    sni_hostname: Option<String>,
    verify_hostname: Option<bool>,
    handshake_timeout_ms: Option<u64>,
    heartbeat_interval_ms: Option<u64>,
    heartbeat_timeout_ms: Option<u64>,
    client_uid: Option<String>,
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
    let (tak_status_tx, tak_status_rx) = watch::channel(tak_client::TakClientStatus::default());
    let state = Arc::new(GatewayState {
        world,
        tx,
        tak_client: Arc::new(Mutex::new(None::<tokio::task::JoinHandle<()>>)),
        tak_status_tx,
        tak_status_rx,
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

fn spawn_tak_client(opts: tak_client::TakClientOptions, state: &GatewayState) {
    let world_client = Arc::clone(&state.world);
    let status_tx = state.tak_status_tx.clone();
    let handle = tokio::spawn(async move {
        tak_client::run_tak_client_loop(opts, world_client, status_tx).await;
    });
    let mut lock = state
        .tak_client
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(prev) = lock.take() {
        prev.abort();
    }
    *lock = Some(handle);
}

#[allow(dead_code)]
fn spawn_tak_tcp_client(addr: String, state: &GatewayState) {
    spawn_tak_client(tak_client::TakClientOptions::new_tcp(addr), state);
}

async fn handle_http_connection(
    mut stream: TcpStream,
    state: Arc<GatewayState>,
) -> Result<(), http::DynError> {
    let Some(req) = http::read_request(&mut stream).await? else {
        return Ok(());
    };
    match router::dispatch(&req, &state).await? {
        router::RouteOutput::Response(resp) => http::write_response(&mut stream, &resp).await,
        router::RouteOutput::SseStream(rx) => http::stream_sse(&mut stream, rx).await,
    }
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
