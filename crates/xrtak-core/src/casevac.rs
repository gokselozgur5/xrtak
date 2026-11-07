// crates/xrtak-core/src/casevac.rs
//! NATO 9-Line MEDEVAC & Helicopter Landing Zone (HLZ) Suitability Engine (ICD-24, REQ-MED-01..02).

use serde::{Deserialize, Serialize};
use crate::geo::ElevationProvider;
use crate::domain::ThreatDome;

/// Patient precedent / urgency level (Line 3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PatientPrecedence {
    Urgent,
    UrgentSurgical,
    Priority,
    Routine,
    Convenience,
}

impl PatientPrecedence {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Urgent => "A - URGENT",
            Self::UrgentSurgical => "B - URGENT-SURG",
            Self::Priority => "C - PRIORITY",
            Self::Routine => "D - ROUTINE",
            Self::Convenience => "E - CONVENIENCE",
        }
    }
}

/// NATO 9-Line MEDEVAC input parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedevacReport {
    pub line1_location: String,
    pub line2_freq_callsign: String,
    pub line3_precedence: PatientPrecedence,
    pub line4_special_equipment: String,
    pub line5_litter_count: u32,
    pub line5_ambulatory_count: u32,
    pub line6_security: String,
    pub line7_hlz_marking: String,
    pub line8_patient_nationality: String,
    pub line9_nbc_or_terrain: String,
    pub lat: f64,
    pub lon: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
}

/// Formatted NATO 9-Line text representation.
pub fn format_nato_nine_line_text(report: &MedevacReport) -> String {
    format!(
        "--- NATO 9-LINE MEDEVAC REQUEST ---\n\
         LINE 1 (Location): {}\n\
         LINE 2 (Freq/Callsign): {}\n\
         LINE 3 (Precedence): {}\n\
         LINE 4 (Special Equip): {}\n\
         LINE 5 (Patients): {} Litter / {} Ambulatory\n\
         LINE 6 (Security): {}\n\
         LINE 7 (Marking): {}\n\
         LINE 8 (Nationality): {}\n\
         LINE 9 (NBC/Terrain): {}",
        report.line1_location,
        report.line2_freq_callsign,
        report.line3_precedence.code(),
        report.line4_special_equipment,
        report.line5_litter_count,
        report.line5_ambulatory_count,
        report.line6_security,
        report.line7_hlz_marking,
        report.line8_patient_nationality,
        report.line9_nbc_or_terrain
    )
}

/// Helicopter Landing Zone (HLZ) terrain and threat evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HlzSuitability {
    pub max_slope_deg: f64,
    pub avg_slope_deg: f64,
    pub is_slope_acceptable: bool,
    pub min_threat_distance_m: f64,
    pub nearest_threat_callsign: Option<String>,
    pub is_threat_clear: bool,
    pub overall_suitability: String,
}

/// Evaluate terrain slope around candidate HLZ radius (m) and safety distance to threat domes.
pub fn evaluate_hlz_suitability(
    center_east_m: f64,
    center_north_m: f64,
    hlz_radius_m: f64,
    elevation: &dyn ElevationProvider,
    threat_domes: &[ThreatDome],
) -> HlzSuitability {
    let radius = hlz_radius_m.max(15.0);
    let center_elev = elevation.elevation_m(center_east_m, center_north_m);

    let mut max_slope: f64 = 0.0;
    let mut sum_slope: f64 = 0.0;
    let radial_samples = 12;

    for i in 0..radial_samples {
        let angle = (i as f64) * (2.0 * std::f64::consts::PI / radial_samples as f64);
        let sample_e = center_east_m + radius * angle.cos();
        let sample_n = center_north_m + radius * angle.sin();
        let sample_elev = elevation.elevation_m(sample_e, sample_n);
        let delta_h = (sample_elev - center_elev).abs();
        let slope_deg = (delta_h / radius).atan().to_degrees();

        if slope_deg > max_slope {
            max_slope = slope_deg;
        }
        sum_slope += slope_deg;
    }

    let avg_slope = sum_slope / (radial_samples as f64);
    let is_slope_acceptable = max_slope <= 8.0;

    let mut min_threat_dist = f64::MAX;
    let mut nearest_threat: Option<String> = None;

    for dome in threat_domes {
        let de = center_east_m - dome.center_enu.0;
        let dn = center_north_m - dome.center_enu.1;
        let dist = (de.powi(2) + dn.powi(2)).sqrt() - dome.radius_m;
        if dist < min_threat_dist {
            min_threat_dist = dist;
            nearest_threat = Some(dome.label.clone());
        }
    }

    let is_threat_clear = min_threat_dist > 500.0;
    let overall_suitability = if is_slope_acceptable && is_threat_clear {
        "GREEN - SUITABLE".to_string()
    } else if is_slope_acceptable && !is_threat_clear {
        "AMBER - THREAT PROXIMITY".to_string()
    } else if !is_slope_acceptable && is_threat_clear {
        "AMBER - STEEP TERRAIN".to_string()
    } else {
        "RED - UNSUITABLE".to_string()
    };

    HlzSuitability {
        max_slope_deg: (max_slope * 10.0).round() / 10.0,
        avg_slope_deg: (avg_slope * 10.0).round() / 10.0,
        is_slope_acceptable,
        min_threat_distance_m: if min_threat_dist == f64::MAX { 99999.0 } else { (min_threat_dist * 10.0).round() / 10.0 },
        nearest_threat_callsign: nearest_threat,
        is_threat_clear,
        overall_suitability,
    }
}
