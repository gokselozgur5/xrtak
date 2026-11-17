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
