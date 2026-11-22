use super::battery::BatteryTelemetry;
use super::connection::ConnectionTelemetry;
use super::types::ConnectionQuality;
use serde::{Deserialize, Serialize};

/// Composite telemetry HUD element payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudTelemetryElement {
    pub battery: BatteryTelemetry,
    pub connection: ConnectionTelemetry,
    pub callsign: String,
}

impl HudTelemetryElement {
    pub fn new(
        callsign: impl Into<String>,
        battery: BatteryTelemetry,
        connection: ConnectionTelemetry,
    ) -> Self {
        Self {
            callsign: callsign.into(),
            battery,
            connection,
        }
    }

    pub fn format_status_line(&self) -> String {
        let conn_str = match self.connection.quality {
            ConnectionQuality::Disconnected => "DISCONNECTED",
            ConnectionQuality::Poor => "POOR",
            ConnectionQuality::Good => "GOOD",
            ConnectionQuality::Excellent => "EXCELLENT",
        };
        format!(
            "[{}] BAT: {:.0}% | LINK: {} ({:.0}ms)",
            self.callsign, self.battery.percentage, conn_str, self.connection.ping_latency_ms
        )
    }
}
