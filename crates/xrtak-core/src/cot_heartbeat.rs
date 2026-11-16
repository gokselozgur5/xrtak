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
