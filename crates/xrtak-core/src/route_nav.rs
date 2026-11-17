//! Tactical route navigation waypoint sequence generator and ETA estimator.

use serde::{Deserialize, Serialize};

/// Earth radius in meters for geodetic calculations.
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculate Haversine surface distance in meters between two coordinates.
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let (r1, r2) = (lat1.to_radians(), lat2.to_radians());
    let a = (d_lat / 2.0).sin().powi(2) + r1.cos() * r2.cos() * (d_lon / 2.0).sin().powi(2);
    EARTH_RADIUS_M * 2.0 * a.sqrt().asin()
}

/// Calculate initial forward azimuth / bearing in degrees [0, 360).
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (r1, r2) = (lat1.to_radians(), lat2.to_radians());
    let d_lon = (lon2 - lon1).to_radians();
    let y = d_lon.sin() * r2.cos();
    let x = r1.cos() * r2.sin() - r1.sin() * r2.cos() * d_lon.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

/// 3D distance accounting for elevation delta in meters.
pub fn slant_distance_3d_m(
    lat1: f64,
    lon1: f64,
    alt1: f64,
    lat2: f64,
    lon2: f64,
    alt2: f64,
) -> f64 {
    let surface_d = haversine_distance_m(lat1, lon1, lat2, lon2);
    let dz = alt2 - alt1;
    (surface_d.powi(2) + dz.powi(2)).sqrt()
}

/// Cross-track error in meters from point P to segment A->B.
/// Positive is right of track; negative is left of track.
pub fn cross_track_error_m(
    plat: f64,
    plon: f64,
    alat: f64,
    alon: f64,
    blat: f64,
    blon: f64,
) -> f64 {
    let d13 = haversine_distance_m(alat, alon, plat, plon) / EARTH_RADIUS_M;
    let brg13 = initial_bearing_deg(alat, alon, plat, plon).to_radians();
    let brg12 = initial_bearing_deg(alat, alon, blat, blon).to_radians();
    let xtd = (d13.sin() * (brg13 - brg12).sin()).asin();
    xtd * EARTH_RADIUS_M
}

/// Tactical waypoint classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaypointType {
    Start,
    TurnPoint,
    RallyPoint,
    Target,
    Checkpoint,
    Ingress,
    Egress,
    End,
}

/// Single navigation waypoint with 3D coordinates and constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavWaypoint {
    pub id: String,
    pub callsign: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
    pub wp_type: WaypointType,
    pub speed_limit_mps: Option<f64>,
}

impl NavWaypoint {
    /// Create a new tactical navigation waypoint.
    pub fn new(
        id: impl Into<String>,
        callsign: impl Into<String>,
        lat: f64,
        lon: f64,
        alt_m: f64,
        wp_type: WaypointType,
    ) -> Self {
        Self {
            id: id.into(),
            callsign: callsign.into(),
            lat,
            lon,
            alt_m,
            wp_type,
            speed_limit_mps: None,
        }
    }
}

impl NavWaypoint {
    /// Attach an optional maximum speed constraint in meters per second.
    pub fn with_speed_limit(mut self, speed_mps: f64) -> Self {
        self.speed_limit_mps = Some(speed_mps);
        self
    }
}

/// Vehicle movement profile defining kinematic limits for ETA estimation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleMovementProfile {
    pub cruise_speed_mps: f64,
    pub max_climb_rate_mps: f64,
    pub max_descent_rate_mps: f64,
}

impl Default for VehicleMovementProfile {
    fn default() -> Self {
        Self {
            cruise_speed_mps: 15.0,
            max_climb_rate_mps: 5.0,
            max_descent_rate_mps: 8.0,
        }
    }
}

impl VehicleMovementProfile {
    pub fn new(cruise_speed_mps: f64, max_climb_rate_mps: f64, max_descent_rate_mps: f64) -> Self {
        Self {
            cruise_speed_mps: cruise_speed_mps.max(0.1),
            max_climb_rate_mps: max_climb_rate_mps.max(0.1),
            max_descent_rate_mps: max_descent_rate_mps.max(0.1),
        }
    }
}

/// Calculated navigation estimates for a single route leg between waypoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegNavEstimate {
    pub leg_index: usize,
    pub from_wp: String,
    pub to_wp: String,
    pub distance_3d_m: f64,
    pub true_bearing_deg: f64,
    pub ete_seconds: f64,
    pub cumulative_distance_m: f64,
    pub cumulative_ete_seconds: f64,
}

/// Tactical route plan containing ordered waypoints and precomputed leg estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalRoutePlan {
    pub route_id: String,
    pub name: String,
    pub waypoints: Vec<NavWaypoint>,
    pub legs: Vec<LegNavEstimate>,
    pub total_distance_m: f64,
    pub total_ete_seconds: f64,
}

/// Compute estimated time en route (ETE) in seconds for a 3D leg based on profile.
pub fn estimate_leg_ete_seconds(
    distance_3d_m: f64,
    delta_alt_m: f64,
    speed_limit: Option<f64>,
    profile: &VehicleMovementProfile,
) -> f64 {
    let speed = speed_limit
        .map(|sl| sl.min(profile.cruise_speed_mps))
        .unwrap_or(profile.cruise_speed_mps);
    let horiz_time = distance_3d_m / speed.max(0.1);
    let vert_time = if delta_alt_m > 0.0 {
        delta_alt_m / profile.max_climb_rate_mps
    } else {
        delta_alt_m.abs() / profile.max_descent_rate_mps
    };
    horiz_time.max(vert_time)
}

impl LegNavEstimate {
    /// Construct a calculated leg navigation estimate.
    pub fn new(
        idx: usize,
        from: &str,
        to: &str,
        d3d: f64,
        brg: f64,
        ete: f64,
        cd: f64,
        ce: f64,
    ) -> Self {
        Self {
            leg_index: idx,
            from_wp: from.to_string(),
            to_wp: to.to_string(),
            distance_3d_m: d3d,
            true_bearing_deg: brg,
            ete_seconds: ete,
            cumulative_distance_m: cd,
            cumulative_ete_seconds: ce,
        }
    }
}

fn build_leg(
    i: usize,
    w1: &NavWaypoint,
    w2: &NavWaypoint,
    prof: &VehicleMovementProfile,
    cd: &mut f64,
    ce: &mut f64,
) -> LegNavEstimate {
    let d3d = slant_distance_3d_m(w1.lat, w1.lon, w1.alt_m, w2.lat, w2.lon, w2.alt_m);
    let brg = initial_bearing_deg(w1.lat, w1.lon, w2.lat, w2.lon);
    let ete = estimate_leg_ete_seconds(d3d, w2.alt_m - w1.alt_m, w2.speed_limit_mps, prof);
    *cd += d3d;
    *ce += ete;
    LegNavEstimate::new(i, &w1.id, &w2.id, d3d, brg, ete, *cd, *ce)
}

impl TacticalRoutePlan {
    /// Return the count of waypoints in this plan.
    pub fn waypoints_count(&self) -> usize {
        self.waypoints.len()
    }

    /// Return true if the route contains no waypoints.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
}

impl TacticalRoutePlan {
    /// Construct TacticalRoutePlan by computing leg metrics.
    pub fn build(id: &str, name: &str, wps: Vec<NavWaypoint>, p: &VehicleMovementProfile) -> Self {
        let (mut legs, mut cd, mut ce) = (Vec::new(), 0.0, 0.0);
        for i in 0..wps.len().saturating_sub(1) {
            legs.push(build_leg(i, &wps[i], &wps[i + 1], p, &mut cd, &mut ce));
        }
        Self {
            route_id: id.to_string(),
            name: name.to_string(),
            waypoints: wps,
            legs,
            total_distance_m: cd,
            total_ete_seconds: ce,
        }
    }
}

/// Tactical navigation turn-by-turn guidance instruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavGuidanceCue {
    pub message: String,
    pub distance_to_turn_m: f64,
    pub relative_turn_bearing_deg: f64,
}

/// Active runtime tracker monitoring progress along a TacticalRoutePlan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteNavTracker {
    pub route: TacticalRoutePlan,
    pub current_leg_index: usize,
    pub acceptance_radius_m: f64,
    pub max_off_track_m: f64,
    pub is_completed: bool,
}

impl RouteNavTracker {
    /// Initialize tracker on a tactical route with default acceptance limits.
    pub fn new(route: TacticalRoutePlan) -> Self {
        Self {
            route,
            current_leg_index: 0,
            acceptance_radius_m: 25.0,
            max_off_track_m: 100.0,
            is_completed: false,
        }
    }
}

/// Real-time progression metrics against current leg and target waypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteNavStatus {
    pub current_leg_index: usize,
    pub target_waypoint_id: String,
    pub distance_to_target_m: f64,
    pub bearing_to_target_deg: f64,
    pub cross_track_error_m: f64,
    pub eta_to_target_seconds: f64,
    pub eta_total_seconds: f64,
    pub is_off_track: bool,
    pub guidance: Option<NavGuidanceCue>,
}

impl RouteNavTracker {
    fn generate_turn_cue(dist: f64, brg: f64, heading: f64) -> Option<NavGuidanceCue> {
        let mut rel = (brg - heading) % 360.0;
        if rel > 180.0 {
            rel -= 360.0;
        }
        if rel < -180.0 {
            rel += 360.0;
        }
        let turn = if rel.abs() < 10.0 {
            "Maintain"
        } else if rel > 0.0 {
            "Right"
        } else {
            "Left"
        };
        Some(NavGuidanceCue {
            message: format!("{turn} {:+.0} deg in {:.0}m", rel, dist),
            distance_to_turn_m: dist,
            relative_turn_bearing_deg: rel,
        })
    }
}

impl RouteNavTracker {
    /// Advance to next waypoint if currently within acceptance radius of target.
    pub fn advance_if_reached(&mut self, dist_to_target_m: f64) -> bool {
        if !self.is_completed && dist_to_target_m <= self.acceptance_radius_m {
            if self.current_leg_index + 1 < self.route.legs.len() {
                self.current_leg_index += 1;
                true
            } else {
                self.is_completed = true;
                true
            }
        } else {
            false
        }
    }
}

impl RouteNavTracker {
    /// Target waypoint ID for the active leg.
    pub fn current_target_waypoint_id(&self) -> Option<&str> {
        self.route
            .legs
            .get(self.current_leg_index)
            .map(|l| l.to_wp.as_str())
    }

    /// Remaining legs count from current position.
    pub fn remaining_legs_count(&self) -> usize {
        self.route.legs.len().saturating_sub(self.current_leg_index)
    }
}

impl RouteNavTracker {
    /// Calculate remaining distance in subsequent legs beyond the current leg.
    pub fn remaining_subsequent_distance_m(&self) -> f64 {
        if self.current_leg_index + 1 < self.route.legs.len() {
            self.route.legs[self.current_leg_index + 1..]
                .iter()
                .map(|l| l.distance_3d_m)
                .sum()
        } else {
            0.0
        }
    }
}

impl RouteNavTracker {
    /// Return the reference (from, to) waypoints for current leg.
    pub fn current_leg_endpoints(&self) -> Option<(&NavWaypoint, &NavWaypoint)> {
        let from = self.route.waypoints.get(self.current_leg_index)?;
        let to = self.route.waypoints.get(self.current_leg_index + 1)?;
        Some((from, to))
    }
}

/// Positional metrics relative to current leg endpoints.
#[derive(Debug, Clone, Copy)]
pub struct PositionMetrics {
    pub dist_m: f64,
    pub brg_deg: f64,
    pub xtd_m: f64,
}

impl PositionMetrics {
    pub fn compute(lat: f64, lon: f64, alt: f64, from: &NavWaypoint, to: &NavWaypoint) -> Self {
        Self {
            dist_m: slant_distance_3d_m(lat, lon, alt, to.lat, to.lon, to.alt_m),
            brg_deg: initial_bearing_deg(lat, lon, to.lat, to.lon),
            xtd_m: cross_track_error_m(lat, lon, from.lat, from.lon, to.lat, to.lon),
        }
    }
}

impl RouteNavStatus {
    /// Construct RouteNavStatus from computed metrics.
    pub fn from_metrics(
        idx: usize,
        tgt: &str,
        pm: PositionMetrics,
        speed: f64,
        rem_d: f64,
        max_xtd: f64,
        cue: Option<NavGuidanceCue>,
    ) -> Self {
        let eta_tgt = pm.dist_m / speed;
        Self {
            current_leg_index: idx,
            target_waypoint_id: tgt.to_string(),
            distance_to_target_m: pm.dist_m,
            bearing_to_target_deg: pm.brg_deg,
            cross_track_error_m: pm.xtd_m,
            eta_to_target_seconds: eta_tgt,
            eta_total_seconds: eta_tgt + (rem_d / speed),
            is_off_track: pm.xtd_m.abs() > max_xtd,
            guidance: cue,
        }
    }
}

impl RouteNavTracker {
    /// Evaluate current position against active route leg.
    pub fn evaluate_position(
        &mut self,
        lat: f64,
        lon: f64,
        alt: f64,
        spd: f64,
        hdg: Option<f64>,
    ) -> Option<RouteNavStatus> {
        let (from, to) = self.current_leg_endpoints()?;
        let pm = PositionMetrics::compute(lat, lon, alt, from, to);
        let cue = hdg.and_then(|h| Self::generate_turn_cue(pm.dist_m, pm.brg_deg, h));
        let rem_d = self.remaining_subsequent_distance_m();
        self.advance_if_reached(pm.dist_m);
        Some(RouteNavStatus::from_metrics(
            self.current_leg_index,
            &to.id,
            pm,
            spd.max(0.1),
            rem_d,
            self.max_off_track_m,
            cue,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_and_bearing() {
        let (lat1, lon1) = (37.7749, -122.4194);
        let (lat2, lon2) = (37.7750, -122.4194);
        let dist = haversine_distance_m(lat1, lon1, lat2, lon2);
        assert!((dist - 11.1).abs() < 1.0);
        let brg = initial_bearing_deg(lat1, lon1, lat2, lon2);
        assert!(brg < 1.0 || brg > 359.0);
    }

    #[test]
    fn test_slant_distance_and_cross_track() {
        let (lat1, lon1, alt1) = (0.0, 0.0, 0.0);
        let (lat2, lon2, alt2) = (0.0, 0.01, 100.0);
        let d = slant_distance_3d_m(lat1, lon1, alt1, lat2, lon2, alt2);
        assert!(d > 1000.0);
        let xtd = cross_track_error_m(0.001, 0.005, lat1, lon1, lat2, lon2);
        assert!(xtd.abs() > 0.0);
    }

    fn sample_route() -> TacticalRoutePlan {
        let p = VehicleMovementProfile::default();
        let w1 = NavWaypoint::new("WP1", "A", 0.0, 0.0, 0.0, WaypointType::Start);
        let w2 = NavWaypoint::new("WP2", "B", 0.0, 0.1, 0.0, WaypointType::End);
        TacticalRoutePlan::build("R1", "Test", vec![w1, w2], &p)
    }

    #[test]
    fn test_route_plan_builder() {
        let plan = sample_route();
        assert_eq!(plan.waypoints_count(), 2);
        assert_eq!(plan.legs.len(), 1);
        assert!(plan.total_distance_m > 10000.0);
        assert!(plan.total_ete_seconds > 0.0);
    }

    #[test]
    fn test_tracker_evaluation_and_guidance() {
        let plan = sample_route();
        let mut tracker = RouteNavTracker::new(plan);
        assert_eq!(tracker.current_target_waypoint_id(), Some("WP2"));
        let st = tracker.evaluate_position(0.0, 0.05, 0.0, 10.0, Some(90.0));
        assert!(st.is_some());
        let s = st.unwrap();
        assert_eq!(s.target_waypoint_id, "WP2");
        assert!(!s.is_off_track);
        assert!(s.guidance.is_some());
    }

    #[test]
    fn test_waypoint_advancement_and_completion() {
        let plan = sample_route();
        let mut tracker = RouteNavTracker::new(plan);
        assert!(!tracker.is_completed);
        let advanced = tracker.advance_if_reached(10.0);
        assert!(advanced);
        assert!(tracker.is_completed);
    }
}
