use super::types::ConnectionQuality;
use serde::{Deserialize, Serialize};

/// Connection and network telemetry sample for HUD rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionTelemetry {
    pub server_connected: bool,
    pub ping_latency_ms: f32,
    pub packet_loss_pct: f32,
    pub quality: ConnectionQuality,
}
