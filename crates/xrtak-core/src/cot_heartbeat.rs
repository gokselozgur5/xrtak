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
