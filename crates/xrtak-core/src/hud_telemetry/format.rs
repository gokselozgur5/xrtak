use super::element::HudTelemetryElement;
use super::types::ConnectionQuality;

impl HudTelemetryElement {
    pub fn format_status_line(&self) -> String {
        let conn_str = match self.connection.quality {
            ConnectionQuality::Disconnected => "DISCONNECTED",
            ConnectionQuality::Poor => "POOR",
            ConnectionQuality::Good => "GOOD",
            ConnectionQuality::Excellent => "EXCELLENT",
        };
        format!(
            "[{}] BAT: {:.0}% | LINK: {} ({:.0}ms)",
            self.callsign,
            self.battery.percentage,
            conn_str,
            self.connection.ping_latency_ms
        )
    }
}
