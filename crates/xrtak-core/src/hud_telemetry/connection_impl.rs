use super::connection::ConnectionTelemetry;
use super::types::ConnectionQuality;

impl ConnectionTelemetry {
    pub fn new(server_connected: bool, ping_latency_ms: f32, packet_loss_pct: f32) -> Self {
        let quality = if !server_connected {
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
