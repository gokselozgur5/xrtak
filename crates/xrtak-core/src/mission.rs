use crate::geo::ElevationProvider;
use crate::GeoOrigin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMissionWaypoint {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
    /// True when `alt_m` is relative to ground/takeoff (AGL) rather than ASL/HAE
    pub alt_is_relative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionWaypoint3d {
    pub index: usize,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub flight_asl_m: f64,
    pub flight_up_m: f64,
    pub terrain_relief_m: f64,
    pub terrain_asl_m: f64,
    pub agl_m: f64,
    pub cumulative_dist_m: f64,
    pub clearance_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCorridorResult {
    pub mission_name: String,
    pub corridor_width_m: f64,
    pub min_safe_agl_m: f64,
    pub total_distance_m: f64,
    pub min_agl_m: f64,
    pub max_asl_m: f64,
    pub violation_count: usize,
    pub waypoints: Vec<MissionWaypoint3d>,
}

/// Projects raw mission waypoints into the active local tangent plane (`GeoOrigin`),
/// samples terrain elevation via `ElevationProvider`, and evaluates AGL clearance (`REQ-MIS-02`).
pub fn evaluate_mission_corridor(
    mission_name: impl Into<String>,
    raw_waypoints: &[RawMissionWaypoint],
    origin: &GeoOrigin,
    terrain: &dyn ElevationProvider,
    corridor_width_m: f64,
    min_safe_agl_m: f64,
) -> MissionCorridorResult {
    let mut waypoints = Vec::with_capacity(raw_waypoints.len());
    let mut cumulative_dist_m = 0.0;
    let mut min_agl_m = f64::INFINITY;
    let mut max_asl_m = f64::NEG_INFINITY;
    let mut violation_count = 0usize;
    let mut prev_en: Option<(f64, f64)> = None;

    for (idx, wp) in raw_waypoints.iter().enumerate() {
        let (east_m, _, north_m) = origin.lla_to_enu(wp.lat, wp.lon, origin.alt_m);
        let terrain_relief_m = terrain.sample_relief_m(east_m, north_m);
        let terrain_asl_m = origin.alt_m + terrain_relief_m;

        let flight_asl_m = if wp.alt_is_relative {
            terrain_asl_m + wp.alt_m
        } else if wp.alt_m <= 0.0 {
            terrain_asl_m + 60.0
        } else {
            wp.alt_m
        };
        let flight_up_m = (flight_asl_m - origin.alt_m).max(0.0);
        let agl_m = flight_asl_m - terrain_asl_m;

        if let Some((pe, pn)) = prev_en {
            cumulative_dist_m += ((east_m - pe).powi(2) + (north_m - pn).powi(2)).sqrt();
        }
        prev_en = Some((east_m, north_m));

        min_agl_m = min_agl_m.min(agl_m);
        max_asl_m = max_asl_m.max(flight_asl_m);
        let clearance_violation = agl_m < min_safe_agl_m;
        if clearance_violation {
            violation_count += 1;
        }

        waypoints.push(MissionWaypoint3d {
            index: idx,
            name: wp.name.clone(),
            lat: wp.lat,
            lon: wp.lon,
            east_m: (east_m * 100.0).round() / 100.0,
            north_m: (north_m * 100.0).round() / 100.0,
            flight_asl_m: (flight_asl_m * 100.0).round() / 100.0,
            flight_up_m: (flight_up_m * 100.0).round() / 100.0,
            terrain_relief_m: (terrain_relief_m * 100.0).round() / 100.0,
            terrain_asl_m: (terrain_asl_m * 100.0).round() / 100.0,
            agl_m: (agl_m * 100.0).round() / 100.0,
            cumulative_dist_m: (cumulative_dist_m * 10.0).round() / 10.0,
            clearance_violation,
        });
    }

    if waypoints.is_empty() {
        min_agl_m = 0.0;
        max_asl_m = origin.alt_m;
    }

    MissionCorridorResult {
        mission_name: mission_name.into(),
        corridor_width_m: corridor_width_m.max(10.0),
        min_safe_agl_m,
        total_distance_m: (cumulative_dist_m * 10.0).round() / 10.0,
        min_agl_m: (min_agl_m * 100.0).round() / 100.0,
        max_asl_m: (max_asl_m * 100.0).round() / 100.0,
        violation_count,
        waypoints,
    }
}
