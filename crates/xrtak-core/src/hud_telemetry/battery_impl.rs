use super::battery::BatteryTelemetry;
use super::types::BatteryState;

pub const CRITICAL_BATTERY_PCT: f32 = 10.0;
pub const LOW_BATTERY_PCT: f32 = 20.0;
pub const FULL_BATTERY_PCT: f32 = 99.0;

impl BatteryTelemetry {
    pub fn new(percentage: f32, voltage_v: f32, is_charging: bool) -> Self {
        let pct = if percentage.is_finite() {
            percentage.clamp(0.0, 100.0)
        } else {
            0.0
        };
        let state = if is_charging {
            if pct >= FULL_BATTERY_PCT {
                BatteryState::Full
            } else {
                BatteryState::Charging
            }
        } else if pct <= CRITICAL_BATTERY_PCT {
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
