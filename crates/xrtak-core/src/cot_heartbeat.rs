//! Cursor-on-Target (CoT) ping/pong heartbeat keepalive manager.

use serde::{Deserialize, Serialize};

/// Heartbeat status of a TAK peer or server connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartbeatStatus {
    /// Actively receiving pongs within expected interval.
    Connected,
    /// Missed one or more pongs, but within tolerance threshold.
    Degraded,
    /// Timed out or exceeded maximum missed ping count.
    Disconnected,
}

/// Configuration parameters for CoT ping/pong heartbeat keepalive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CotHeartbeatConfig {
    /// Interval between ping heartbeat broadcasts in milliseconds.
    pub interval_ms: u64,
    /// Timeout to wait for a pong response before considering it lost.
    pub timeout_ms: u64,
    /// Maximum consecutive missed pongs before marking connection Disconnected.
    pub max_missed_pongs: u32,
    /// Unique client identifier for CoT event attribution.
    pub client_uid: String,
}

impl Default for CotHeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 15_000,
            timeout_ms: 5_000,
            max_missed_pongs: 3,
            client_uid: "XRTAK-CLIENT".to_string(),
        }
    }
}

/// Record of an active ping transmitted and awaiting pong acknowledgement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingRecord {
    pub seq: u64,
    pub ping_uid: String,
    pub sent_unix_ms: u64,
}

/// Action determined by processing an incoming CoT message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatAction {
    /// Incoming message was a ping; respond with generated pong CoT XML.
    RespondWithPong(String),
    /// Incoming message was a pong acknowledging a pending ping.
    PongAcknowledged { seq: u64, rtt_ms: u64 },
    /// Message is not a heartbeat event or did not match pending pings.
    Ignored,
}

/// State machine managing CoT ping/pong keepalive cycles and latency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CotHeartbeatManager {
    pub config: CotHeartbeatConfig,
    pub(crate) seq: u64,
    pub(crate) last_ping_sent_ms: Option<u64>,
    pub(crate) last_pong_received_ms: Option<u64>,
    pub(crate) last_rtt_ms: Option<u64>,
    pub(crate) smoothed_rtt_ms: f32,
    pub(crate) consecutive_missed: u32,
    pub(crate) total_pings_sent: u64,
    pub(crate) total_pongs_received: u64,
    pub(crate) total_pongs_lost: u64,
    pub(crate) pending_pings: Vec<PingRecord>,
}

impl Default for CotHeartbeatManager {
    fn default() -> Self {
        Self::new(CotHeartbeatConfig::default())
    }
}

impl CotHeartbeatManager {
    /// Create a new heartbeat manager with specified configuration.
    pub fn new(config: CotHeartbeatConfig) -> Self {
        Self {
            config,
            seq: 0,
            last_ping_sent_ms: None,
            last_pong_received_ms: None,
            last_rtt_ms: None,
            smoothed_rtt_ms: 0.0,
            consecutive_missed: 0,
            total_pings_sent: 0,
            total_pongs_received: 0,
            total_pongs_lost: 0,
            pending_pings: Vec::new(),
        }
    }
}
