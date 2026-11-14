use super::battery::BatteryTelemetry;
use super::connection::ConnectionTelemetry;
use super::element::HudTelemetryElement;

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
}
