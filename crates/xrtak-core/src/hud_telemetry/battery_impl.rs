use super::battery::BatteryTelemetry;
use super::types::BatteryState;

impl BatteryTelemetry {
    pub fn new(percentage: f32, voltage_v: f32, is_charging: bool) -> Self {
        let pct = percentage.clamp(0.0, 100.0);
        let state = if is_charging {
            if pct >= 99.0 {
                BatteryState::Full
            } else {
                BatteryState::Charging
            }
        } else if pct <= 15.0 {
            BatteryState::Critical
        } else {
            BatteryState::Discharging
        };

        Self {
            percentage: pct,
            voltage_v,
            state,
        }
    }
}
