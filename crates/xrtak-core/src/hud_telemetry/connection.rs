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

impl ConnectionTelemetry {
    pub fn new(server_connected: bool, ping_latency_ms: f32, packet_loss_pct: f32) -> Self {
        let quality =
            if !server_connected || !ping_latency_ms.is_finite() || !packet_loss_pct.is_finite() {
                ConnectionQuality::Disconnected
            } else if ping_latency_ms > 250.0 || packet_loss_pct > 15.0 {
                ConnectionQuality::Poor
            } else if ping_latency_ms > 80.0 || packet_loss_pct > 2.0 {
                ConnectionQuality::Good
            } else {
                ConnectionQuality::Excellent
            };

        Self {
            server_connected,
            ping_latency_ms,
            packet_loss_pct,
            quality,
        }
    }
}
