use super::battery::BatteryTelemetry;
use super::battery_impl::{CRITICAL_BATTERY_PCT, LOW_BATTERY_PCT};

impl BatteryTelemetry {
    pub fn is_low(&self) -> bool {
        self.percentage <= LOW_BATTERY_PCT
    }

    pub fn is_critical(&self) -> bool {
        self.percentage <= CRITICAL_BATTERY_PCT
    }
}
