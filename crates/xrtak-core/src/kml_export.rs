//! KML document exporter for CoT event tracks.

use serde::{Deserialize, Serialize};

/// WGS84 kinematic position point along a tactical track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KmlTrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub timestamp_ms: Option<u64>,
    pub speed_mps: Option<f64>,
    pub course_deg: Option<f64>,
}

impl KmlTrackPoint {
    pub fn new(lat: f64, lon: f64, hae_m: f64) -> Self {
        Self {
            lat,
            lon,
            hae_m,
            timestamp_ms: None,
            speed_mps: None,
            course_deg: None,
        }
    }

    pub fn with_time(mut self, timestamp_ms: u64) -> Self {
        self.timestamp_ms = Some(timestamp_ms);
        self
    }

    pub fn with_kinematics(mut self, speed_mps: f64, course_deg: f64) -> Self {
        self.speed_mps = Some(speed_mps);
        self.course_deg = Some(course_deg);
        self
    }
}

/// Great-circle Haversine distance between two WGS84 coordinates in meters.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + r_lat1.cos() * r_lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}

/// CoT tactical event track history for KML document generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CotTrack {
    pub uid: String,
    pub callsign: String,
    pub cot_type: String,
    pub points: Vec<KmlTrackPoint>,
}

impl CotTrack {
    pub fn new(
        uid: impl Into<String>,
        callsign: impl Into<String>,
        cot_type: impl Into<String>,
    ) -> Self {
        Self {
            uid: uid.into(),
            callsign: callsign.into(),
            cot_type: cot_type.into(),
            points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, point: KmlTrackPoint) {
        self.points.push(point);
    }

    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn duration_sec(&self) -> Option<f64> {
        let first = self.points.first()?.timestamp_ms?;
        let last = self.points.last()?.timestamp_ms?;
        if last >= first {
            Some((last - first) as f64 / 1000.0)
        } else {
            None
        }
    }

    pub fn path_length_m(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for window in self.points.windows(2) {
            total += haversine_distance_m(
                window[0].lat,
                window[0].lon,
                window[1].lat,
                window[1].lon,
            );
        }
        total
    }
}

/// Resolves KML aabbggrr hex color from CoT type string.
pub fn cot_type_to_kml_color(cot_type: &str) -> &'static str {
    let lower = cot_type.to_lowercase();
    if lower.starts_with("a-f") {
        "ff00ff00"
    } else if lower.starts_with("a-h") {
        "ff0000ff"
    } else if lower.starts_with("a-n") {
        "ffff0000"
    } else if lower.starts_with("a-u") {
        "ff00ffff"
    } else {
        "ffffffff"
    }
}

/// Converts Unix millisecond timestamp to ISO 8601 UTC string.
pub fn format_iso8601_utc(unix_ms: u64) -> String {
    let total_secs = unix_ms / 1000;
    let millis = unix_ms % 1000;

    let sec = total_secs % 60;
    let total_mins = total_secs / 60;
    let min = total_mins % 60;
    let total_hours = total_mins / 60;
    let hour = total_hours % 24;
    let mut days = total_hours / 24;

    let mut year = 1970u64;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
