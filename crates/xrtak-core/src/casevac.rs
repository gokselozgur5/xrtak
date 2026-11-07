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
