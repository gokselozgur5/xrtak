use serde::{Deserialize, Serialize};

/// Battery charging and power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryState {
    Discharging,
    Charging,
    Full,
    Critical,
}

/// Link connection status for TAK and mesh networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionQuality {
    Disconnected,
    Poor,
    Good,
    Excellent,
}
