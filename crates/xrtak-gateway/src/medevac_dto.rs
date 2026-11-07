use serde::{Deserialize, Serialize};
use xrtak_core::casevac::{HlzSuitability, MedevacReport};

#[derive(Debug, Deserialize)]
pub struct MedevacNineLineRequest {
    pub location: String,
    pub freq_callsign: String,
    pub precedence: Option<String>,
    pub special_equipment: Option<String>,
    pub litter_count: Option<u32>,
    pub ambulatory_count: Option<u32>,
    pub security: Option<String>,
    pub marking: Option<String>,
    pub nationality: Option<String>,
    pub terrain_remarks: Option<String>,
    pub enu_center: (f64, f64, f64),
    pub hlz_radius_m: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct MedevacNineLineResponse {
    pub formatted_nine_line: String,
    pub report: MedevacReport,
    pub hlz_suitability: HlzSuitability,
}
