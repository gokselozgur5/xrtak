//! Tactical Radio Communications Frequency and Channel Allocation CoT packet builder.
//!
//! Complies with ATAK radio channel definitions and tactical net plans.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Tactical radio modulation and transmission mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadioMode {
    Am,
    Fm,
    NarrowbandFm,
    FrequencyHopping,
    Digital,
}

impl RadioMode {
    pub fn as_str(&self) -> &'static str {
        match self {
