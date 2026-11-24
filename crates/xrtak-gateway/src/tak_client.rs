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
