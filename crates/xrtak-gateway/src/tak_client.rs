//! TAK client connection manager supporting plaintext TCP, TLS/mTLS, auto-reconnect, and CoT heartbeat keepalive.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncWrite};
use xrtak_core::cot_heartbeat::HeartbeatStatus;

pub type AsyncByteReader = Box<dyn AsyncRead + Unpin + Send>;
pub type AsyncByteWriter = Box<dyn AsyncWrite + Unpin + Send>;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn load_pem_or_string(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.contains("-----BEGIN") {
        trimmed.to_string()
    } else if let Ok(content) = std::fs::read_to_string(Path::new(trimmed)) {
        content
    } else {
        trimmed.to_string()
    }
}

/// Live operational status of the gateway TAK client connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakClientStatus {
    pub configured: bool,
    pub connected: bool,
    pub target_addr: Option<String>,
    pub is_tls: bool,
    pub is_mtls: bool,
    pub reconnect_attempts: u64,
    pub heartbeat_status: HeartbeatStatus,
    pub smoothed_rtt_ms: f32,
    pub last_rtt_ms: Option<u64>,
    pub consecutive_missed_heartbeats: u32,
}

impl Default for TakClientStatus {
    fn default() -> Self {
        Self {
            configured: false,
            connected: false,
            target_addr: None,
            is_tls: false,
            is_mtls: false,
            reconnect_attempts: 0,
            heartbeat_status: HeartbeatStatus::Disconnected,
            smoothed_rtt_ms: 0.0,
            last_rtt_ms: None,
            consecutive_missed_heartbeats: 0,
        }
    }
}

/// Configuration options for the gateway TAK client session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakClientOptions {
    pub target_addr: String,
    pub tls_enabled: bool,
    pub ca_cert_pem: Option<String>,
    pub client_cert_pem: Option<String>,
    pub client_key_pem: Option<String>,
    pub sni_hostname: Option<String>,
    pub verify_hostname: Option<bool>,
    pub handshake_timeout_ms: Option<u64>,
    pub heartbeat_interval_ms: Option<u64>,
    pub heartbeat_timeout_ms: Option<u64>,
    pub client_uid: Option<String>,
}

impl TakClientOptions {
    pub fn new_tcp(addr: impl Into<String>) -> Self {
        Self {
            target_addr: addr.into(),
            tls_enabled: false,
            ca_cert_pem: None,
            client_cert_pem: None,
            client_key_pem: None,
            sni_hostname: None,
            verify_hostname: None,
            handshake_timeout_ms: None,
            heartbeat_interval_ms: None,
            heartbeat_timeout_ms: None,
            client_uid: None,
        }
    }
}

use rustls::pki_types::CertificateDer;
use rustls::{ClientConfig, RootCertStore};
use std::io::Cursor;
use xrtak_core::tak_tls::{ClientAuthMode, TakTlsConfig};

pub fn build_client_config(
    config: &TakTlsConfig,
    client_key_pem: Option<&str>,
) -> Result<ClientConfig, String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut root_store = RootCertStore::empty();
    if let Some(ca) = &config.ca_cert {
        let mut reader = Cursor::new(&ca.raw_data);
        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("invalid ca cert pem: {e}"))?;
        for c in certs {
            root_store
                .add(c)
                .map_err(|e| format!("failed to add root cert: {e}"))?;
        }
    }

    let builder = ClientConfig::builder().with_root_certificates(root_store);
    if config.client_auth == ClientAuthMode::Required {
        let client_cert = config
            .client_cert
            .as_ref()
            .ok_or_else(|| "missing client cert for mtls".to_string())?;
        let mut cert_reader = Cursor::new(&client_cert.raw_data);
        let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("invalid client cert pem: {e}"))?;
        let key_raw = client_key_pem.ok_or_else(|| "missing client key pem for mtls".to_string())?;
        let key_pem = load_pem_or_string(key_raw);
        let mut key_reader = Cursor::new(key_pem.as_bytes());
        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| format!("invalid client key pem: {e}"))?
            .ok_or_else(|| "no private key found in client key pem".to_string())?;
        builder
            .with_client_auth_cert(certs, key)
            .map_err(|e| format!("client auth config error: {e}"))
    } else {
        Ok(builder.with_no_client_auth())
    }
}

use rustls::pki_types::ServerName;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use xrtak_core::tak_tls::TlsCertificate;

pub fn build_tls_context(
    options: &TakClientOptions,
) -> Result<Option<(Arc<ClientConfig>, TakTlsConfig)>, String> {
    if !options.tls_enabled {
        return Ok(None);
    }
    let host_port: Vec<&str> = options.target_addr.split(':').collect();
    let host = host_port.first().copied().unwrap_or("127.0.0.1");
    let port: u16 = host_port.get(1).and_then(|p| p.parse().ok()).unwrap_or(8089);
    let mut builder = TakTlsConfig::builder(host, port);
    if let Some(ca) = &options.ca_cert_pem {
        let ca_pem = load_pem_or_string(ca);
        builder = builder.ca_cert(TlsCertificate::from_pem(ca_pem.as_bytes().to_vec()));
    }
    if let Some(cert) = &options.client_cert_pem {
        let cert_pem = load_pem_or_string(cert);
        builder = builder
            .client_cert(TlsCertificate::from_pem(cert_pem.as_bytes().to_vec()))
            .client_auth(ClientAuthMode::Required);
    }
    if let Some(sni) = &options.sni_hostname {
        builder = builder.sni_hostname(sni);
    }
    if let Some(verify) = options.verify_hostname {
        builder = builder.verify_server_hostname(verify);
    }
    if let Some(to) = options.handshake_timeout_ms {
        builder = builder.handshake_timeout_ms(to);
    }
    let cfg = builder.build().map_err(|e| format!("invalid tls config: {e}"))?;
    let client_cfg = build_client_config(&cfg, options.client_key_pem.as_deref())?;
    Ok(Some((Arc::new(client_cfg), cfg)))
}

pub async fn establish_stream(
    target_addr: &str,
    tls_pair: Option<&(Arc<ClientConfig>, TakTlsConfig)>,
) -> Result<(AsyncByteReader, AsyncByteWriter), String> {
    let tcp = TcpStream::connect(target_addr.trim())
        .await
        .map_err(|e| format!("tcp connect failed: {e}"))?;
    if let Some((client_cfg, cfg)) = tls_pair {
        let connector = TlsConnector::from(Arc::clone(client_cfg));
        let sni_str = cfg.sni_hostname.as_deref().unwrap_or(&cfg.server_host);
        let server_name = ServerName::try_from(sni_str)
            .map_err(|e| format!("invalid sni hostname: {e}"))?
            .to_owned();
        let tls_stream = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| format!("tls handshake failed: {e}"))?;
        let (tr, tw) = tokio::io::split(tls_stream);
        Ok((Box::new(tr), Box::new(tw)))
    } else {
        let (tr, tw) = tokio::io::split(tcp);
        Ok((Box::new(tr), Box::new(tw)))
    }
}

use std::sync::{Mutex, PoisonError};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::watch;
use xrtak_core::cot_heartbeat::{CotHeartbeatConfig, CotHeartbeatManager, HeartbeatAction};
use xrtak_core::TacticalWorld;

pub async fn run_session_stream(
    mut reader: AsyncByteReader,
    mut writer: AsyncByteWriter,
    world: Arc<Mutex<TacticalWorld>>,
    status_tx: &watch::Sender<TakClientStatus>,
    hb_cfg: CotHeartbeatConfig,
) {
    let mut hb = CotHeartbeatManager::new(hb_cfg);
    let mut buf = vec![0u8; 65535];
    let mut carry = String::new();
    let tick_interval = Duration::from_millis(500);

    loop {
        let now = now_ms();
        if hb.should_send_ping(now) {
            let (ping_xml, _seq) = hb.generate_ping(now);
            if writer.write_all(ping_xml.as_bytes()).await.is_err() {
                break;
            }
        }
        hb.check_timeouts(now);
        status_tx.send_modify(|s| {
            s.heartbeat_status = hb.status();
            s.smoothed_rtt_ms = hb.smoothed_rtt_ms();
            s.last_rtt_ms = hb.last_rtt_ms();
            s.consecutive_missed_heartbeats = hb.consecutive_missed();
        });
        if !hb.is_connected() && hb.consecutive_missed() >= hb.config.max_missed_pongs {
            eprintln!("[XR-TAK] Heartbeat timeout exceeded, triggering reconnect");
            break;
        }

        tokio::select! {
            read_res = reader.read(&mut buf) => {
                let n = match read_res {
                    Ok(0) => break,
                    Ok(bytes_read) => bytes_read,
                    Err(_) => break,
                };
                carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                while let Some(end_idx) = carry.find("</event>") {
                    let end = end_idx + "</event>".len();
                    let segment = carry[..end].to_string();
                    carry = carry[end..].to_string();
                    if let Some(start) = segment.find("<event") {
                        let xml = &segment[start..];
                        let cur_time = now_ms();
                        let action = hb.process_incoming_cot(xml, cur_time);
                        match action {
                            HeartbeatAction::RespondWithPong(pong_xml) => {
                                if writer.write_all(pong_xml.as_bytes()).await.is_err() {
                                    return;
                                }
                            }
                            HeartbeatAction::PongAcknowledged { .. } => {
                                status_tx.send_modify(|s| {
                                    s.heartbeat_status = hb.status();
                                    s.smoothed_rtt_ms = hb.smoothed_rtt_ms();
                                    s.last_rtt_ms = hb.last_rtt_ms();
                                    s.consecutive_missed_heartbeats = hb.consecutive_missed();
                                });
                            }
                            HeartbeatAction::Ignored => {
                                let mut guard = world.lock().unwrap_or_else(PoisonError::into_inner);
                                let _ = guard.ingest_cot_xml(xml, cur_time);
                            }
                        }
                    }
                }
            }
            _ = tokio::time::sleep(tick_interval) => {}
        }
    }
}

pub async fn run_tak_client_loop(
    options: TakClientOptions,
    world: Arc<Mutex<TacticalWorld>>,
    status_tx: watch::Sender<TakClientStatus>,
) {
    let mut reconnect_attempts = 0u64;
    let mut backoff = Duration::from_millis(500);
    let max_backoff = Duration::from_secs(10);

    let tls_config = match build_tls_context(&options) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("[XR-TAK TLS] Failed to initialize TLS context: {err}");
            return;
        }
    };

    let is_tls = tls_config.is_some();
    let is_mtls = tls_config.as_ref().map(|(_, c)| c.is_mtls()).unwrap_or(false);

    loop {
        status_tx.send_modify(|s| {
            s.configured = true;
            s.connected = false;
            s.target_addr = Some(options.target_addr.clone());
            s.is_tls = is_tls;
            s.is_mtls = is_mtls;
            s.reconnect_attempts = reconnect_attempts;
        });

        match establish_stream(&options.target_addr, tls_config.as_ref()).await {
            Ok((r, w)) => {
                backoff = Duration::from_millis(500);
                let hb_cfg = CotHeartbeatConfig {
                    interval_ms: options.heartbeat_interval_ms.unwrap_or(15_000),
                    timeout_ms: options.heartbeat_timeout_ms.unwrap_or(5_000),
                    max_missed_pongs: 3,
                    client_uid: options
                        .client_uid
                        .clone()
                        .unwrap_or_else(|| "XRTAK-GATEWAY".to_string()),
                };

                status_tx.send_modify(|s| {
                    s.connected = true;
                    s.heartbeat_status = HeartbeatStatus::Connected;
                });

                run_session_stream(r, w, Arc::clone(&world), &status_tx, hb_cfg).await;

                status_tx.send_modify(|s| {
                    s.connected = false;
                    s.heartbeat_status = HeartbeatStatus::Disconnected;
                });
            }
            Err(e) => {
                eprintln!("[XR-TAK] Connection failed to {}: {e}", options.target_addr);
            }
        }

        reconnect_attempts += 1;
        status_tx.send_modify(|s| {
            s.connected = false;
            s.reconnect_attempts = reconnect_attempts;
        });
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(max_backoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use xrtak_core::GeoOrigin;

    #[tokio::test]
    async fn test_tak_client_auto_reconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let world = Arc::new(Mutex::new(TacticalWorld::new(GeoOrigin::default())));
        let (status_tx, mut status_rx) = watch::channel(TakClientStatus::default());

        let options = TakClientOptions::new_tcp(addr);
        let client_world = Arc::clone(&world);
        let client_tx = status_tx.clone();
        let handle = tokio::spawn(async move {
            run_tak_client_loop(options, client_world, client_tx).await;
        });

        let (mut s1, _) = listener.accept().await.unwrap();
        s1.write_all(b"<event version=\"2.0\" uid=\"RECON-1\" type=\"a-f-G\" how=\"m-g\" time=\"2026-11-24T00:00:00Z\" start=\"2026-11-24T00:00:00Z\" stale=\"2026-11-24T00:05:00Z\"><point lat=\"10.0\" lon=\"20.0\" hae=\"0.0\" ce=\"10.0\" le=\"10.0\"/><detail/></event>").await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        {
            let guard = world.lock().unwrap();
            assert!(guard.entities.contains_key("RECON-1"));
        }

        drop(s1);

        let (mut s2, _) = listener.accept().await.unwrap();
        s2.write_all(b"<event version=\"2.0\" uid=\"RECON-2\" type=\"a-f-G\" how=\"m-g\" time=\"2026-11-24T00:00:00Z\" start=\"2026-11-24T00:00:00Z\" stale=\"2026-11-24T00:05:00Z\"><point lat=\"10.1\" lon=\"20.1\" hae=\"0.0\" ce=\"10.0\" le=\"10.0\"/><detail/></event>").await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        {
            let guard = world.lock().unwrap();
            assert!(guard.entities.contains_key("RECON-2"));
        }

        status_rx.changed().await.ok();
        let status = status_rx.borrow().clone();
        assert!(status.connected);
        assert!(status.reconnect_attempts >= 1);

        handle.abort();
    }

    #[tokio::test]
    async fn test_cot_heartbeat_roundtrip_and_rtt() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let world = Arc::new(Mutex::new(TacticalWorld::new(GeoOrigin::default())));
        let (status_tx, mut status_rx) = watch::channel(TakClientStatus::default());

        let mut options = TakClientOptions::new_tcp(addr);
        options.heartbeat_interval_ms = Some(100);
        options.heartbeat_timeout_ms = Some(500);

        let client_world = Arc::clone(&world);
        let client_tx = status_tx.clone();
        let handle = tokio::spawn(async move {
            run_tak_client_loop(options, client_world, client_tx).await;
        });

        let (mut server_stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 4096];

        let n = server_stream.read(&mut buf).await.unwrap();
        let xml = String::from_utf8_lossy(&buf[..n]);
        assert!(xml.contains("type=\"t-x-c-t\""));
        assert!(xml.contains("XRTAK-GATEWAY-ping-1"));

        let pong = r#"<event version="2.0" uid="TAK-pong-1" type="t-x-c-t-r" how="m-g" time="2026-11-24T00:00:00Z" start="2026-11-24T00:00:00Z" stale="2026-11-24T00:05:00Z"><point lat="0.0" lon="0.0" hae="0.0" ce="999" le="999"/><detail><pong seq="1" ref_uid="XRTAK-GATEWAY-ping-1"/></detail></event>"#;
        server_stream.write_all(pong.as_bytes()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;
        status_rx.changed().await.ok();
        let status = status_rx.borrow().clone();
        assert!(status.last_rtt_ms.is_some());
        assert_eq!(status.heartbeat_status, HeartbeatStatus::Connected);

        let server_ping = r#"<event version="2.0" uid="SERVER-ping-42" type="t-x-c-t" how="m-g" time="2026-11-24T00:00:00Z" start="2026-11-24T00:00:00Z" stale="2026-11-24T00:05:00Z"><point lat="0.0" lon="0.0" hae="0.0" ce="999" le="999"/><detail><ping seq="42" client="SERVER"/></detail></event>"#;
        server_stream.write_all(server_ping.as_bytes()).await.unwrap();

        let n2 = server_stream.read(&mut buf).await.unwrap();
        let xml2 = String::from_utf8_lossy(&buf[..n2]);
        assert!(xml2.contains("type=\"t-x-c-t-r\""));
        assert!(xml2.contains("ref_uid=\"SERVER-ping-42\""));

        handle.abort();
    }
}
