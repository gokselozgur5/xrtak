use super::battery::BatteryTelemetry;

impl BatteryTelemetry {
    pub fn is_low(&self) -> bool {
        self.percentage <= 20.0
    }

    pub fn is_critical(&self) -> bool {
        self.percentage <= 10.0
    }
}
