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

/// Point-to-point 3D Line-of-Sight (intervisibility) result (`REQ-TAC-01`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkLosResult {
    pub from_enu: (f64, f64, f64),
    pub to_enu: (f64, f64, f64),
    pub slant_range_m: f64,
    pub clear: bool,
    pub min_clearance_m: f64,
    pub obstruction_enu: Option<(f64, f64, f64)>,
}

/// Volumetric 3D SAM / Radar Threat Dome (`REQ-TAC-02`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDome {
    pub uid: String,
    pub callsign: String,
    pub affiliation: Affiliation,
    pub east_m: f64,
    pub up_m: f64,
    pub north_m: f64,
    pub radius_m: f64,
    pub ceiling_m: f64,
    /// UIDs of airborne/ground tracks currently inside the dome with unobstructed radar LOS
    pub engaged_uids: Vec<String>,
}

/// 3D Polygonal Geofence / Restricted Operating Zone (`ROZ`, `Killbox`, `u-d-f` CoT shape) (`REQ-TAC-03`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalZone {
    pub uid: String,
    pub name: String,
    pub affiliation: Affiliation,
    /// Ordered 2D polygon vertices in local `(east_m, north_m)` coordinates
    pub vertices_en: Vec<(f64, f64)>,
    pub floor_m: f64,
    pub ceiling_m: f64,
    /// UIDs of entities currently inside this 3D polygonal prism
    pub inside_uids: Vec<String>,
}
