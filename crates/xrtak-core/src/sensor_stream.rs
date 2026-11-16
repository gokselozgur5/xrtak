//! TAK sensor video stream descriptor XML generator and parser.
//!
//! Provides Cursor-on-Target (CoT) `b-i-v` XML event generation and
//! TAK video connection `<feed>` descriptor XML serialization.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during sensor video stream descriptor XML operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorStreamError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingDetailTag,
    MissingVideoTag,
    MissingFeedTag,
    MissingAttribute(&'static str),
}

impl fmt::Display for SensorStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {}", e),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingDetailTag => write!(f, "Missing detail tag"),
            Self::MissingVideoTag => write!(f, "Missing video stream tag"),
            Self::MissingFeedTag => write!(f, "Missing feed tag"),
            Self::MissingAttribute(a) => write!(f, "Missing attribute: {}", a),
        }
    }
}

impl std::error::Error for SensorStreamError {}

/// Type of optical or imaging sensor payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorType {
    ElectroOptical,
    Infrared,
    ShortWaveInfrared,
    MidWaveInfrared,
    LongWaveInfrared,
    DualEoIr,
    SyntheticApertureRadar,
    Other(String),
}

impl SensorType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ElectroOptical => "EO",
            Self::Infrared => "IR",
            Self::ShortWaveInfrared => "SWIR",
            Self::MidWaveInfrared => "MWIR",
            Self::LongWaveInfrared => "LWIR",
            Self::DualEoIr => "EO/IR",
            Self::SyntheticApertureRadar => "SAR",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl SensorType {
    pub fn from_str_name(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "EO" => Self::ElectroOptical,
            "IR" => Self::Infrared,
            "SWIR" => Self::ShortWaveInfrared,
            "MWIR" => Self::MidWaveInfrared,
            "LWIR" => Self::LongWaveInfrared,
            "EO/IR" | "EOIR" => Self::DualEoIr,
            "SAR" => Self::SyntheticApertureRadar,
            other => Self::Other(other.to_string()),
        }
    }
}

impl Default for SensorType {
    fn default() -> Self {
        Self::ElectroOptical
    }
}

/// Optics and gimbal orientation telemetry of a sensor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorGimbal {
    pub sensor_type: SensorType,
    pub model: String,
    pub fov_horizontal_deg: f64,
    pub fov_vertical_deg: f64,
    pub slant_range_m: f64,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub roll_deg: f64,
    pub zoom_ratio: f64,
}

impl Default for SensorGimbal {
    fn default() -> Self {
        Self {
            sensor_type: SensorType::ElectroOptical,
            model: "GENERIC-EO".to_string(),
            fov_horizontal_deg: 45.0,
            fov_vertical_deg: 30.0,
            slant_range_m: 1000.0,
            azimuth_deg: 0.0,
            elevation_deg: -45.0,
            roll_deg: 0.0,
            zoom_ratio: 1.0,
        }
    }
}
