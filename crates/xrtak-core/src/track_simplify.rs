//! CoT track history polyline simplification using Ramer-Douglas-Peucker (RDP).
//!
//! Complies with REQ-TRK-03 and ICD-20 for track history compression and ATAK breadcrumbs.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

/// WGS84 mean earth radius in meters.
pub const WGS84_EARTH_RADIUS_M: f64 = 6_371_000.0;

/// WGS84 kinematic position point along a tactical entity track history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub timestamp_ms: Option<u64>,
    pub speed_mps: Option<f64>,
    pub course_deg: Option<f64>,
}

impl TrackPoint {
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

/// Configuration parameters for track polyline simplification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSimplifyConfig {
    pub epsilon_m: f64,
    pub include_elevation: bool,
    pub max_time_gap_sec: Option<f64>,
}

impl Default for TrackSimplifyConfig {
    fn default() -> Self {
        Self {
            epsilon_m: 5.0,
            include_elevation: false,
            max_time_gap_sec: None,
        }
    }
}

/// Result of track polyline simplification including compression statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSimplifyResult {
    pub points: Vec<TrackPoint>,
    pub original_count: usize,
    pub simplified_count: usize,
    pub reduction_ratio: f64,
    pub max_error_m: f64,
}

/// Great-circle Haversine distance between two WGS84 coordinates in meters.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + r_lat1.cos() * r_lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    WGS84_EARTH_RADIUS_M * c
}

/// Initial true bearing from coordinate 1 to coordinate 2 in degrees [0, 360).
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let y = d_lon.sin() * r_lat2.cos();
    let x = r_lat1.cos() * r_lat2.sin() - r_lat1.sin() * r_lat2.cos() * d_lon.cos();
    let brg = y.atan2(x).to_degrees();
    (brg + 360.0) % 360.0
}

/// Computes perpendicular distance from point P to line segment A->B in meters.
pub fn perpendicular_distance_to_segment_m(
    p: &TrackPoint,
    a: &TrackPoint,
    b: &TrackPoint,
    include_elevation: bool,
) -> f64 {
    let seg_dist = haversine_distance_m(a.lat, a.lon, b.lat, b.lon);
    if seg_dist < 1e-6 {
        let d = haversine_distance_m(p.lat, p.lon, a.lat, a.lon);
        return if include_elevation {
            (d.powi(2) + (p.hae_m - a.hae_m).powi(2)).sqrt()
        } else {
            d
        };
    }

    let d_ap = haversine_distance_m(a.lat, a.lon, p.lat, p.lon);
    let brg_ab = initial_bearing_deg(a.lat, a.lon, b.lat, b.lon).to_radians();
    let brg_ap = initial_bearing_deg(a.lat, a.lon, p.lat, p.lon).to_radians();
    let delta_brg = brg_ap - brg_ab;

    let along_dist = d_ap * delta_brg.cos();
    if along_dist <= 0.0 {
        let d = haversine_distance_m(p.lat, p.lon, a.lat, a.lon);
        return if include_elevation {
            (d.powi(2) + (p.hae_m - a.hae_m).powi(2)).sqrt()
        } else {
            d
        };
    }

    if along_dist >= seg_dist {
        let d = haversine_distance_m(p.lat, p.lon, b.lat, b.lon);
        return if include_elevation {
            (d.powi(2) + (p.hae_m - b.hae_m).powi(2)).sqrt()
        } else {
            d
        };
    }

    let cross_dist = (d_ap * delta_brg.sin()).abs();
    if include_elevation {
        let t = along_dist / seg_dist;
        let interp_hae = a.hae_m + t * (b.hae_m - a.hae_m);
        let dz = p.hae_m - interp_hae;
        (cross_dist.powi(2) + dz.powi(2)).sqrt()
    } else {
        cross_dist
    }
}

fn rdp_recursive(
    points: &[TrackPoint],
    epsilon_m: f64,
    include_elevation: bool,
    max_err: &mut f64,
) -> Vec<TrackPoint> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let first = &points[0];
    let last = &points[points.len() - 1];

    let mut max_dist = 0.0;
    let mut split_idx = 0usize;

    for (i, pt) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let dist = perpendicular_distance_to_segment_m(pt, first, last, include_elevation);
        if dist > max_dist {
            max_dist = dist;
            split_idx = i;
        }
    }

    if max_dist > *max_err {
        *max_err = max_dist;
    }

    if max_dist > epsilon_m {
        let mut left = rdp_recursive(&points[..=split_idx], epsilon_m, include_elevation, max_err);
        let right = rdp_recursive(&points[split_idx..], epsilon_m, include_elevation, max_err);
        left.pop();
        left.extend(right);
        left
    } else {
        vec![first.clone(), last.clone()]
    }
}

/// Simplifies a track point slice using the Ramer-Douglas-Peucker algorithm.
pub fn simplify_track_rdp(points: &[TrackPoint], epsilon_m: f64) -> Vec<TrackPoint> {
    let config = TrackSimplifyConfig {
        epsilon_m,
        ..Default::default()
    };
    simplify_track_with_config(points, &config).points
}

/// Simplifies a slice of 2D coordinates (lat, lon) using Ramer-Douglas-Peucker.
pub fn simplify_coordinates_rdp(coords: &[(f64, f64)], epsilon_m: f64) -> Vec<(f64, f64)> {
    if coords.len() <= 2 {
        return coords.to_vec();
    }
    let pts: Vec<TrackPoint> = coords
        .iter()
        .map(|&(lat, lon)| TrackPoint::new(lat, lon, 0.0))
        .collect();
    let simplified = simplify_track_rdp(&pts, epsilon_m);
    simplified.into_iter().map(|p| (p.lat, p.lon)).collect()
}

/// Simplifies a track according to configuration parameters and returns summary metrics.
pub fn simplify_track_with_config(
    points: &[TrackPoint],
    config: &TrackSimplifyConfig,
) -> TrackSimplifyResult {
    let orig_count = points.len();
    if orig_count <= 2 {
        return TrackSimplifyResult {
            points: points.to_vec(),
            original_count: orig_count,
            simplified_count: orig_count,
            reduction_ratio: 0.0,
            max_error_m: 0.0,
        };
    }

    let mut max_err = 0.0f64;

    let simplified_points = if let Some(gap_sec) = config.max_time_gap_sec {
        let gap_ms = (gap_sec * 1000.0) as u64;
        let mut segments: Vec<Vec<TrackPoint>> = Vec::new();
        let mut current_segment: Vec<TrackPoint> = vec![points[0].clone()];

        for i in 1..points.len() {
            let prev = &points[i - 1];
            let curr = &points[i];
            let is_gap = match (prev.timestamp_ms, curr.timestamp_ms) {
                (Some(t0), Some(t1)) => t1.saturating_sub(t0) > gap_ms,
                _ => false,
            };
            if is_gap {
                segments.push(current_segment);
                current_segment = vec![curr.clone()];
            } else {
                current_segment.push(curr.clone());
            }
        }
        segments.push(current_segment);

        let mut combined: Vec<TrackPoint> = Vec::new();
        for seg in segments {
            let seg_res = rdp_recursive(&seg, config.epsilon_m, config.include_elevation, &mut max_err);
            if combined.is_empty() {
                combined.extend(seg_res);
            } else {
                combined.extend(seg_res.into_iter().skip(1));
            }
        }
        combined
    } else {
        rdp_recursive(points, config.epsilon_m, config.include_elevation, &mut max_err)
    };

    let simp_count = simplified_points.len();
    let ratio = if orig_count > 0 {
        1.0 - (simp_count as f64 / orig_count as f64)
    } else {
        0.0
    };

    TrackSimplifyResult {
        points: simplified_points,
        original_count: orig_count,
        simplified_count: simp_count,
        reduction_ratio: ratio,
        max_error_m: (max_err * 100.0).round() / 100.0,
    }
}

/// Generates ATAK CoT Track History breadcrumb XML (`b-t-f`) for simplified track.
pub fn generate_cot_track_history_xml(uid: &str, callsign: &str, points: &[TrackPoint]) -> String {
    let (first_lat, first_lon, first_hae) = if let Some(p) = points.first() {
        (p.lat, p.lon, p.hae_m)
    } else {
        (0.0, 0.0, 0.0)
    };

    let mut crumbs = String::new();
    for (i, p) in points.iter().enumerate() {
        let time_attr = match p.timestamp_ms {
            Some(t) => format!(r#" time="{}""#, t),
            None => String::new(),
        };
        let speed_attr = match p.speed_mps {
            Some(s) => format!(r#" speed="{:.2}""#, s),
            None => String::new(),
        };
        let course_attr = match p.course_deg {
            Some(c) => format!(r#" course="{:.1}""#, c),
            None => String::new(),
        };

        crumbs.push_str(&format!(
            r#"<crumb idx="{}" lat="{:.6}" lon="{:.6}" hae="{:.1}"{}{}{}/>"#,
            i, p.lat, p.lon, p.hae_m, time_attr, speed_attr, course_attr
        ));
    }

    format!(
        r#"<event version="2.0" uid="{uid}" type="b-t-f" how="m-g"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae:.1}" ce="10.0" le="10.0"/><detail><contact callsign="{callsign}"/><track_history count="{count}">{crumbs}</track_history></detail></event>"#,
        uid = uid,
        callsign = callsign,
        lat = first_lat,
        lon = first_lon,
        hae = first_hae,
        count = points.len(),
        crumbs = crumbs,
    )
}

/// Parses track points from an ATAK CoT Track History (`b-t-f`) XML document.
pub fn parse_cot_track_history_xml(xml: &str) -> Result<Vec<TrackPoint>, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut points = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"crumb" => {
                    let mut lat = 0.0f64;
                    let mut lon = 0.0f64;
                    let mut hae = 0.0f64;
                    let mut time_ms: Option<u64> = None;
                    let mut speed: Option<f64> = None;
                    let mut course: Option<f64> = None;
