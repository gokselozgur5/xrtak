use serde::{Deserialize, Serialize};
use xrtak_core::fires::BallisticSolution;

#[derive(Debug, Deserialize)]
pub struct BallisticsRequest {
    /// Origin in local tangent plane (East, Up, North) in meters.
    pub origin_enu: (f64, f64, f64),
    /// Target in local tangent plane (East, Up, North) in meters.
    pub target_enu: (f64, f64, f64),
    pub munition: Option<String>,
    pub elevation_angle_deg: Option<f64>,
    pub samples: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct BallisticsResponse {
    pub solution: BallisticSolution,
}
