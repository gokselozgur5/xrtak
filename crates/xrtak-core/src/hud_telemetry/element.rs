use super::battery::BatteryTelemetry;
use super::connection::ConnectionTelemetry;
use serde::{Deserialize, Serialize};

/// Composite telemetry HUD element payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudTelemetryElement {
    pub battery: BatteryTelemetry,
    pub connection: ConnectionTelemetry,
    pub callsign: String,
}
