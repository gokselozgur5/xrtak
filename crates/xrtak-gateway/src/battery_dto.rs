use serde::{Deserialize, Serialize};
use xrtak_core::battery::{UavBatteryState, UavEnduranceSolution, WindVector};

#[derive(Debug, Deserialize)]
pub struct UavEnduranceRequest {
    pub state: UavBatteryState,
    pub wind: Option<WindVector>,
    pub radial_steps: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct UavEnduranceResponse {
    pub solution: UavEnduranceSolution,
}
