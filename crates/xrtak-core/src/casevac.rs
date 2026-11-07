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
