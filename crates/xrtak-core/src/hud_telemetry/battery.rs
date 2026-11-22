use super::types::BatteryState;
use serde::{Deserialize, Serialize};

pub const CRITICAL_BATTERY_PCT: f32 = 10.0;
pub const LOW_BATTERY_PCT: f32 = 20.0;
pub const FULL_BATTERY_PCT: f32 = 99.0;

/// Battery telemetry sample for HUD rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BatteryTelemetry {
    pub percentage: f32,
    pub voltage_v: f32,
    pub state: BatteryState,
}

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

    pub fn is_low(&self) -> bool {
        self.percentage <= LOW_BATTERY_PCT
    }

    pub fn is_critical(&self) -> bool {
        self.percentage <= CRITICAL_BATTERY_PCT
    }
}
