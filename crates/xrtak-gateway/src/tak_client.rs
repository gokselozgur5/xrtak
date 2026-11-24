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
