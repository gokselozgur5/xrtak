use super::types::BatteryState;
use serde::{Deserialize, Serialize};

/// Battery telemetry sample for HUD rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BatteryTelemetry {
    pub percentage: f32,
    pub voltage_v: f32,
    pub state: BatteryState,
}
