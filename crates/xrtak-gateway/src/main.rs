use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::broadcast;
use xrtak_core::{compute_viewshed, GeoOrigin, TacticalEntity, TacticalWorld};

const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Debug, Clone, Serialize)]
struct WorldSnapshot {
    origin: GeoOrigin,
    sim_time_sec: f64,
    entities: Vec<TacticalEntity>,
}

#[derive(Debug, Deserialize)]
struct SpawnRequest {
    callsign: String,
    cot_type: String,
    east_m: f64,
    north_m: f64,
    alt_offset_m: Option<f64>,
    remarks: Option<String>,
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
    let http_addr: SocketAddr = "0.0.0.0:8090".parse()?;
    let udp_cot_addr: SocketAddr = "0.0.0.0:4242".parse()?;

    let world = Arc::new(Mutex::new(TacticalWorld::new(GeoOrigin::default())));
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
                    guard.step_simulation(0.05, now_unix_ms());
                    let mut entities: Vec<TacticalEntity> =
                        guard.entities.values().cloned().collect();
                    entities.sort_by(|a, b| a.uid.cmp(&b.uid));
                    let snap = WorldSnapshot {
                        origin: guard.origin,
                        sim_time_sec: guard.sim_time_sec,
                        entities,
                    };
                    serde_json::to_string(&snap).unwrap_or_default()
                };
                let _ = tx_sim.send(payload);
            }
        });
    }

    // 3. HTTP + SSE + CoT/SPI/Viewshed Server on :8090
    let listener = TcpListener::bind(http_addr).await?;
    println!("==============================================================");
    println!("  XR-TAK — Rust CoT Core + WebXR 3D Tactical Table");
    println!("  WebXR / Browser UI : http://localhost:8090");
    println!("  Raw CoT UDP Input  : udp://0.0.0.0:4242");
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
    let mut buf = vec![0u8; 16384];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0];
    let path = parts[1];

    if method == "GET" && (path == "/" || path == "/index.html") {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
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
                sim_time_sec: guard.sim_time_sec,
                entities,
            };
            serde_json::to_string(&snap)?
        };
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        return Ok(());
    }

    if method == "GET" && path == "/api/stream" {
        let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
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
            guard.ingest_cot_xml(body, now_unix_ms())
        };
        match res {
            Ok(ent) => {
                let json = serde_json::to_string(&ent)?;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    json.len()
                );
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
            }
            Err(err) => {
                let msg = serde_json::json!({ "error": err }).to_string();
                let header = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
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
                let json = serde_json::json!({ "status": "cleared", "sensor_uid": sensor_uid }).to_string();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    json.len()
                );
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
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    json.len()
                );
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    if method == "POST" && path == "/api/viewshed" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(vs_req) = serde_json::from_str::<ViewshedRequest>(body) {
            let vs = compute_viewshed(
                vs_req.east_m,
                vs_req.north_m,
                vs_req.height_offset_m.unwrap_or(18.0),
                vs_req.radius_m.unwrap_or(680.0),
                72,
                28,
            );
            let json = serde_json::to_string(&vs)?;
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                json.len()
            );
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(json.as_bytes()).await?;
            return Ok(());
        }
    }

    if method == "POST" && path == "/api/spawn" {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Ok(spawn) = serde_json::from_str::<SpawnRequest>(body) {
            let ent = {
                let mut guard = world.lock().unwrap();
                let ground_h = xrtak_core::sample_terrain_relief_m(spawn.east_m, spawn.north_m);
                let up_m = ground_h + spawn.alt_offset_m.unwrap_or(0.0);
                let (lat, lon, hae) = guard.origin.enu_to_lla(spawn.east_m, up_m, spawn.north_m);
                let uid = format!(
                    "XR-{}-{}",
                    spawn.callsign.replace(' ', "-"),
                    now_unix_ms() % 10000
                );
                let remarks = spawn
                    .remarks
                    .unwrap_or_else(|| "Dropped from XR Tactical Sand Table".to_string());
                let xml = format!(
                    r#"<event version="2.0" uid="{uid}" type="{cot_type}" how="h-g-i-g-o"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae:.1}" ce="10" le="10"/><detail><contact callsign="{callsign}"/><track course="0.0" speed="0.0"/><remarks>{remarks}</remarks></detail></event>"#,
                    cot_type = spawn.cot_type,
                    callsign = spawn.callsign
                );
                guard.ingest_cot_xml(&xml, now_unix_ms())
            };
            if let Ok(entity) = ent {
                let json = serde_json::to_string(&entity)?;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    json.len()
                );
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(json.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    let not_found =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
    stream.write_all(not_found.as_bytes()).await?;
    Ok(())
}
