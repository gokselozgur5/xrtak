//! CoT route plan waypoint sequencer and ETA estimator.

use serde::{Deserialize, Serialize};

const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Great-circle Haversine distance between two WGS84 positions in meters.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + r_lat1.cos() * r_lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_M * c
}

/// 3D slant distance taking elevation difference into account.
pub fn slant_distance_3d_m(lat1: f64, lon1: f64, alt1: f64, lat2: f64, lon2: f64, alt2: f64) -> f64 {
    let h_dist = haversine_distance_m(lat1, lon1, lat2, lon2);
    let v_dist = alt2 - alt1;
    (h_dist.powi(2) + v_dist.powi(2)).sqrt()
}

/// Initial true bearing from coordinate 1 to 2 in degrees [0, 360).
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_lat1 = lat1.to_radians();
    let r_lat2 = lat2.to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let y = d_lon.sin() * r_lat2.cos();
    let x = r_lat1.cos() * r_lat2.sin() - r_lat1.sin() * r_lat2.cos() * d_lon.cos();
    let brg = y.atan2(x).to_degrees();
    (brg + 360.0) % 360.0
}

/// Cross-track error in meters from point P to geodesic track segment A->B.
/// Positive value indicates right of track; negative indicates left of track.
pub fn cross_track_error_m(p_lat: f64, p_lon: f64, a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let d13 = haversine_distance_m(a_lat, a_lon, p_lat, p_lon) / EARTH_RADIUS_M;
    let brg13 = initial_bearing_deg(a_lat, a_lon, p_lat, p_lon).to_radians();
    let brg12 = initial_bearing_deg(a_lat, a_lon, b_lat, b_lon).to_radians();

    let xt = (d13.sin() * (brg13 - brg12).sin()).asin();
    xt * EARTH_RADIUS_M
}

/// Along-track distance in meters from start waypoint A toward B for point P.
pub fn along_track_distance_m(p_lat: f64, p_lon: f64, a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let d13 = haversine_distance_m(a_lat, a_lon, p_lat, p_lon) / EARTH_RADIUS_M;
    let d_xt = cross_track_error_m(p_lat, p_lon, a_lat, a_lon, b_lat, b_lon) / EARTH_RADIUS_M;
    let at = (d13.cos() / d_xt.cos()).acos();
    if at.is_nan() {
        0.0
    } else {
        at * EARTH_RADIUS_M
    }
}

/// Planned mission waypoint representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteWaypoint {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub planned_speed_mps: Option<f64>,
    pub loiter_sec: f64,
}

impl RouteWaypoint {
    pub fn new(name: impl Into<String>, lat: f64, lon: f64, hae_m: f64) -> Self {
        Self {
            name: name.into(),
            lat,
            lon,
            hae_m,
            planned_speed_mps: None,
            loiter_sec: 0.0,
        }
    }

    pub fn with_speed(mut self, speed_mps: f64) -> Self {
        self.planned_speed_mps = Some(speed_mps);
        self
    }

    pub fn with_loiter(mut self, loiter_sec: f64) -> Self {
        self.loiter_sec = loiter_sec;
        self
    }
}

/// Sequenced leg between consecutive waypoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequencedLeg {
    pub from_index: usize,
    pub to_index: usize,
    pub from_name: String,
    pub to_name: String,
    pub distance_m: f64,
    pub slant_distance_m: f64,
    pub bearing_deg: f64,
    pub speed_mps: f64,
    pub transit_time_sec: f64,
    pub loiter_time_sec: f64,
    pub total_leg_time_sec: f64,
    pub cumulative_dist_m: f64,
    pub cumulative_ete_sec: f64,
    pub eta_epoch_sec: f64,
}

/// Route plan specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutePlan {
    pub route_uid: String,
    pub callsign: String,
    pub default_speed_mps: f64,
    pub start_time_epoch_sec: f64,
    pub waypoints: Vec<RouteWaypoint>,
}

/// Computed sequence and timing schedule for a route plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequencedRouteResult {
    pub route_uid: String,
    pub callsign: String,
    pub total_distance_m: f64,
    pub total_duration_sec: f64,
    pub estimated_completion_epoch_sec: f64,
    pub legs: Vec<SequencedLeg>,
}

/// Sequences waypoints along a route plan, calculating distances, bearings, and ETAs.
pub fn sequence_route_plan(plan: &RoutePlan) -> SequencedRouteResult {
    let mut legs = Vec::new();
    let mut cum_dist = 0.0;
    let mut cum_time = 0.0;
    let default_spd = if plan.default_speed_mps > 0.1 {
        plan.default_speed_mps
    } else {
        15.0
    };

    for i in 1..plan.waypoints.len() {
        let prev = &plan.waypoints[i - 1];
        let curr = &plan.waypoints[i];

        let h_dist = haversine_distance_m(prev.lat, prev.lon, curr.lat, curr.lon);
        let s_dist = slant_distance_3d_m(prev.lat, prev.lon, prev.hae_m, curr.lat, curr.lon, curr.hae_m);
        let brg = initial_bearing_deg(prev.lat, prev.lon, curr.lat, curr.lon);

        let spd = curr.planned_speed_mps.unwrap_or(default_spd).max(0.1);
        let transit_time = s_dist / spd;
        let loiter_time = curr.loiter_sec;
        let leg_time = transit_time + loiter_time;

        cum_dist += s_dist;
        cum_time += leg_time;

        legs.push(SequencedLeg {
            from_index: i - 1,
            to_index: i,
            from_name: prev.name.clone(),
            to_name: curr.name.clone(),
            distance_m: (h_dist * 10.0).round() / 10.0,
            slant_distance_m: (s_dist * 10.0).round() / 10.0,
            bearing_deg: (brg * 10.0).round() / 10.0,
            speed_mps: spd,
            transit_time_sec: (transit_time * 10.0).round() / 10.0,
            loiter_time_sec: loiter_time,
            total_leg_time_sec: (leg_time * 10.0).round() / 10.0,
            cumulative_dist_m: (cum_dist * 10.0).round() / 10.0,
            cumulative_ete_sec: (cum_time * 10.0).round() / 10.0,
            eta_epoch_sec: ((plan.start_time_epoch_sec + cum_time) * 10.0).round() / 10.0,
        });
    }

    SequencedRouteResult {
        route_uid: plan.route_uid.clone(),
        callsign: plan.callsign.clone(),
        total_distance_m: (cum_dist * 10.0).round() / 10.0,
        total_duration_sec: (cum_time * 10.0).round() / 10.0,
        estimated_completion_epoch_sec: ((plan.start_time_epoch_sec + cum_time) * 10.0).round() / 10.0,
        legs,
    }
}

/// Active tracking status along a sequenced route leg.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveTrackProgress {
    pub active_leg_index: usize,
    pub next_waypoint_name: String,
    pub distance_to_next_m: f64,
    pub bearing_to_next_deg: f64,
    pub cross_track_error_m: f64,
    pub time_to_go_sec: f64,
    pub next_wp_eta_epoch_sec: f64,
    pub remaining_route_dist_m: f64,
    pub remaining_route_time_sec: f64,
    pub has_reached_waypoint: bool,
}

/// Evaluates progress of an entity along the active route leg.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_route_progress(
    plan: &RoutePlan,
    sequenced: &SequencedRouteResult,
    active_leg_idx: usize,
    cur_lat: f64,
    cur_lon: f64,
    cur_speed_mps: f64,
    cur_time_sec: f64,
    capture_radius_m: f64,
) -> Option<ActiveTrackProgress> {
    if active_leg_idx >= sequenced.legs.len() {
        return None;
    }

    let leg = &sequenced.legs[active_leg_idx];
    let to_wp = &plan.waypoints[leg.to_index];
    let from_wp = &plan.waypoints[leg.from_index];

    let dist_to_next = haversine_distance_m(cur_lat, cur_lon, to_wp.lat, to_wp.lon);
    let brg_to_next = initial_bearing_deg(cur_lat, cur_lon, to_wp.lat, to_wp.lon);
    let xte = cross_track_error_m(cur_lat, cur_lon, from_wp.lat, from_wp.lon, to_wp.lat, to_wp.lon);

    let spd = if cur_speed_mps > 0.5 {
        cur_speed_mps
    } else {
        leg.speed_mps
    };

    let ttg = dist_to_next / spd;
    let next_eta = cur_time_sec + ttg;

    let mut remaining_dist = dist_to_next;
    let mut remaining_time = ttg;
    for future_leg in &sequenced.legs[active_leg_idx + 1..] {
        remaining_dist += future_leg.slant_distance_m;
        remaining_time += future_leg.total_leg_time_sec;
    }

    let reached = dist_to_next <= capture_radius_m;

    Some(ActiveTrackProgress {
        active_leg_index: active_leg_idx,
        next_waypoint_name: to_wp.name.clone(),
        distance_to_next_m: (dist_to_next * 10.0).round() / 10.0,
        bearing_to_next_deg: (brg_to_next * 10.0).round() / 10.0,
        cross_track_error_m: (xte * 10.0).round() / 10.0,
        time_to_go_sec: (ttg * 10.0).round() / 10.0,
        next_wp_eta_epoch_sec: (next_eta * 10.0).round() / 10.0,
        remaining_route_dist_m: (remaining_dist * 10.0).round() / 10.0,
        remaining_route_time_sec: (remaining_time * 10.0).round() / 10.0,
        has_reached_waypoint: reached,
    })
}

/// Generates ATAK CoT Route XML (b-m-r) from a sequenced route result and plan.
pub fn generate_cot_route_xml(plan: &RoutePlan, sequenced: &SequencedRouteResult) -> String {
    let (first_lat, first_lon, first_hae) = if let Some(first_wp) = plan.waypoints.first() {
        (first_wp.lat, first_wp.lon, first_wp.hae_m)
    } else {
        (0.0, 0.0, 0.0)
    };

    let mut links = String::new();
