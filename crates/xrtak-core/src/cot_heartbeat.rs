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

impl CotHeartbeatManager {
    /// Return the current operational status of the heartbeat keepalive.
    pub fn status(&self) -> HeartbeatStatus {
        if self.consecutive_missed >= self.config.max_missed_pongs {
            HeartbeatStatus::Disconnected
        } else if self.consecutive_missed > 0 {
            HeartbeatStatus::Degraded
        } else if self.last_pong_received_ms.is_some() {
            HeartbeatStatus::Connected
        } else {
            HeartbeatStatus::Disconnected
        }
    }

    /// Check if peer link is currently considered active and connected.
    pub fn is_connected(&self) -> bool {
        self.status() == HeartbeatStatus::Connected
    }
}

impl CotHeartbeatManager {
    /// Return the exponentially smoothed round-trip time in milliseconds.
    pub fn smoothed_rtt_ms(&self) -> f32 {
        self.smoothed_rtt_ms
    }

    /// Return the most recent instantaneous round-trip time in milliseconds.
    pub fn last_rtt_ms(&self) -> Option<u64> {
        self.last_rtt_ms
    }

    /// Return packet loss percentage based on total sent and lost pongs.
    pub fn packet_loss_pct(&self) -> f32 {
        if self.total_pings_sent == 0 {
            0.0
        } else {
            (self.total_pongs_lost as f32 / self.total_pings_sent as f32) * 100.0
        }
    }
}

impl CotHeartbeatManager {
    /// Return the count of currently unacknowledged pending pings.
    pub fn pending_count(&self) -> usize {
        self.pending_pings.len()
    }

    /// Return the consecutive number of missed pong timeouts.
    pub fn consecutive_missed(&self) -> u32 {
        self.consecutive_missed
    }
}

/// Build a standard TAK Cursor-on-Target ping event XML (`t-x-c-t`).
pub fn build_ping_cot(client_uid: &str, seq: u64, now_ms: u64) -> String {
    let uid = format!("{client_uid}-ping-{seq}");
    let stale_ms = now_ms.saturating_add(10_000);
    format!(
        r#"<event version="2.0" uid="{uid}" type="t-x-c-t" how="m-g" time="{now_ms}" start="{now_ms}" stale="{stale_ms}"><point lat="0.000000" lon="0.000000" hae="0.0" ce="9999999" le="9999999"/><detail><ping seq="{seq}" client="{client_uid}"/></detail></event>"#
    )
}

/// Build a standard TAK Cursor-on-Target pong response XML (`t-x-c-t-r`).
pub fn build_pong_cot(client_uid: &str, ping_uid: &str, seq: u64, now_ms: u64) -> String {
    let uid = format!("{client_uid}-pong-{seq}");
    let stale_ms = now_ms.saturating_add(10_000);
    format!(
        r#"<event version="2.0" uid="{uid}" type="t-x-c-t-r" how="m-g" time="{now_ms}" start="{now_ms}" stale="{stale_ms}"><point lat="0.000000" lon="0.000000" hae="0.0" ce="9999999" le="9999999"/><detail><pong seq="{seq}" ref_uid="{ping_uid}"/></detail></event>"#
    )
}

impl CotHeartbeatManager {
    /// Check whether elapsed duration requires sending another ping.
    pub fn should_send_ping(&self, now_ms: u64) -> bool {
        match self.last_ping_sent_ms {
            None => true,
            Some(last) => now_ms.saturating_sub(last) >= self.config.interval_ms,
        }
    }

    /// Generate an outgoing ping event and queue record in pending list.
    pub fn generate_ping(&mut self, now_ms: u64) -> (String, u64) {
        self.seq = self.seq.saturating_add(1);
        let seq = self.seq;
        let ping_uid = format!("{}-ping-{seq}", self.config.client_uid);
        let xml = build_ping_cot(&self.config.client_uid, seq, now_ms);
        self.pending_pings.push(PingRecord {
            seq,
            ping_uid,
            sent_unix_ms: now_ms,
        });
        self.last_ping_sent_ms = Some(now_ms);
        self.total_pings_sent = self.total_pings_sent.saturating_add(1);
        (xml, seq)
    }
}
