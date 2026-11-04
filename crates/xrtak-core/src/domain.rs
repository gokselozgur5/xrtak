use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Affiliation {
    Friendly,
    Hostile,
    Neutral,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dimension {
    Air,
    Ground,
    Surface,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCone {
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub hfov_deg: f64,
    pub vfov_deg: f64,
    pub range_m: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ViewshedSample {
    pub east_m: f64,
    pub up_m: f64,
    pub north_m: f64,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewshedResult {
    pub observer_east_m: f64,
    pub observer_up_m: f64,
    pub observer_north_m: f64,
    pub radius_m: f64,
    pub visible_ratio: f64,
    pub samples: Vec<ViewshedSample>,
}
