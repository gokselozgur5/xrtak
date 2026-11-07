use serde::{Deserialize, Serialize};
use xrtak_core::ew::{BurnThroughAssessment, JammerProfile, JammingAssessment, TargetLinkProfile};

#[derive(Debug, Deserialize)]
pub struct EwJammingRequest {
    pub jammer: JammerProfile,
    pub link: TargetLinkProfile,
    pub band: Option<String>,
    pub jamming_margin_threshold_db: Option<f64>,
    pub radar_burn_through: Option<RadarBurnThroughInput>,
}

#[derive(Debug, Deserialize)]
pub struct RadarBurnThroughInput {
    pub radar_power_w: f64,
    pub radar_gain_dbi: f64,
    pub target_rcs_m2: f64,
    pub current_target_range_m: f64,
}

#[derive(Debug, Serialize)]
pub struct EwJammingResponse {
    pub assessment: JammingAssessment,
    pub burn_through: Option<BurnThroughAssessment>,
}
