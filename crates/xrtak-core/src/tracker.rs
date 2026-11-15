//! Dead-Reckoning, Kinematic Track Extrapolation & CEP Uncertainty Ring Engine.
//!
//! Complies with REQ-TRK-01..02 and ICD-20 for stale/coasting tactical tracks.

use serde::{Deserialize, Serialize};

/// Projected kinematic waypoint at lookahead delta time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPredictionPoint {
    pub dt_sec: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
    pub cep_radius_m: f64,
}

/// Predicted trajectory and confidence boundary for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPrediction {
    pub uid: String,
    pub callsign: String,
    pub current_enu: (f64, f64, f64),
    pub speed_mps: f64,
    pub course_deg: f64,
    pub is_stale: bool,
    pub points: Vec<TrackPredictionPoint>,
}

/// Request to project tracks over a set of lookahead horizons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPredictRequest {
    pub horizons_sec: Option<Vec<f64>>,
    pub stale_threshold_sec: Option<f64>,
}

/// Extrapolates 3D ENU position after dt_sec given velocity vector and heading.
pub fn compute_kinematic_extrapolation(
    current_enu: (f64, f64, f64),
    speed_mps: f64,
    course_deg: f64,
    vert_speed_mps: f64,
    dt_sec: f64,
) -> (f64, f64, f64) {
    let rad = course_deg.to_radians();
    let dx = speed_mps * rad.sin() * dt_sec;
    let dy = speed_mps * rad.cos() * dt_sec;
    let dz = vert_speed_mps * dt_sec;
    (current_enu.0 + dx, current_enu.1 + dy, current_enu.2 + dz)
}

/// Computes expanding Circular Error Probable (CEP) confidence radius in meters.
pub fn compute_cep_radius_m(initial_ce_m: f64, speed_mps: f64, dt_sec: f64) -> f64 {
    let r0 = initial_ce_m.max(2.0);
    let sigma_v = 0.5; // velocity uncertainty m/s
    let sigma_omega = 0.03; // turn rate standard deviation rad/s
    let vel_variance = (sigma_v * dt_sec).powi(2);
    let turn_variance = 0.25 * (sigma_omega * speed_mps * dt_sec.powi(2)).powi(2);
    (r0.powi(2) + vel_variance + turn_variance).sqrt()
}

/// Kinematic prediction input parameters for an entity trajectory projection.
#[derive(Debug, Clone)]
pub struct TrackPredictInput<'a> {
    pub uid: &'a str,
    pub callsign: &'a str,
    pub current_enu: (f64, f64, f64),
    pub speed_mps: f64,
    pub course_deg: f64,
    pub is_stale: bool,
    pub initial_ce_m: f64,
    pub horizons: &'a [f64],
}

impl<'a> TrackPredictInput<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        uid: &'a str,
        callsign: &'a str,
        current_enu: (f64, f64, f64),
        speed_mps: f64,
        course_deg: f64,
        is_stale: bool,
        initial_ce_m: f64,
        horizons: &'a [f64],
    ) -> Self {
        Self {
            uid,
            callsign,
            current_enu,
            speed_mps,
            course_deg,
            is_stale,
            initial_ce_m,
            horizons,
        }
    }

    pub fn predict(&self) -> TrackPrediction {
        predict_entity_trajectory(
            self.uid,
            self.callsign,
            self.current_enu,
            self.speed_mps,
            self.course_deg,
            self.is_stale,
            self.initial_ce_m,
            self.horizons,
        )
    }
}

/// Generates kinematic track prediction points across specified time horizons.
// TODO(XR-070): replace the argument list with a TrackPredictInput struct.
#[allow(clippy::too_many_arguments)]
pub fn predict_entity_trajectory(
    uid: &str,
    callsign: &str,
    current_enu: (f64, f64, f64),
    speed_mps: f64,
    course_deg: f64,
    is_stale: bool,
    initial_ce_m: f64,
    horizons: &[f64],
) -> TrackPrediction {
    let mut points = Vec::new();
    for &dt in horizons {
        let (e, n, u) =
            compute_kinematic_extrapolation(current_enu, speed_mps, course_deg, 0.0, dt);
        let cep = compute_cep_radius_m(initial_ce_m, speed_mps, dt);
        points.push(TrackPredictionPoint {
            dt_sec: dt,
            east_m: (e * 100.0).round() / 100.0,
            north_m: (n * 100.0).round() / 100.0,
            up_m: (u * 100.0).round() / 100.0,
            cep_radius_m: (cep * 100.0).round() / 100.0,
        });
    }

    TrackPrediction {
        uid: uid.to_string(),
        callsign: callsign.to_string(),
        current_enu,
        speed_mps,
        course_deg,
        is_stale,
        points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinematic_extrapolation_cardinal_headings() {
        // Heading 090 (East), speed 20 m/s for 30s -> dx = +600m, dy = 0m
        let origin = (0.0, 0.0, 100.0);
        let pt_east = compute_kinematic_extrapolation(origin, 20.0, 90.0, 0.0, 30.0);
        assert!((pt_east.0 - 600.0).abs() < 1e-3);
        assert!(pt_east.1.abs() < 1e-3);
        assert!((pt_east.2 - 100.0).abs() < 1e-3);

        // Heading 000 (North), speed 10 m/s for 10s -> dx = 0m, dy = +100m
        let pt_north = compute_kinematic_extrapolation(origin, 10.0, 0.0, 0.0, 10.0);
        assert!(pt_north.0.abs() < 1e-3);
        assert!((pt_north.1 - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_expanding_cep_growth() {
        let origin = (0.0, 0.0, 100.0);
        let cep_0 = compute_cep_radius_m(5.0, 25.0, 0.0);
        assert_eq!(cep_0, 5.0);

        let cep_30 = compute_cep_radius_m(5.0, 25.0, 30.0);
        let cep_60 = compute_cep_radius_m(5.0, 25.0, 60.0);
        assert!(cep_30 > cep_0, "CEP at 30s must exceed initial CEP");
        assert!(cep_60 > cep_30, "CEP at 60s must exceed CEP at 30s");

        let pred = predict_entity_trajectory(
            "UAV-01",
            "FALCON",
            origin,
            20.0,
            45.0,
            true,
            5.0,
            &[15.0, 30.0, 60.0],
        );
        assert_eq!(pred.points.len(), 3);
        assert!(pred.is_stale);
    }

    #[test]
    fn test_track_predict_input_predict() {
        let horizons = [10.0, 20.0];
        let input = TrackPredictInput::new(
            "UAV-02",
            "HAWK",
            (100.0, 200.0, 50.0),
            15.0,
            90.0,
            false,
            3.0,
            &horizons,
        );
        let pred = input.predict();
        assert_eq!(pred.uid, "UAV-02");
        assert_eq!(pred.callsign, "HAWK");
        assert!(!pred.is_stale);
        assert_eq!(pred.points.len(), 2);
    }

    #[test]
    fn test_kinematic_extrapolation_vertical_and_headings() {
        let origin = (100.0, 100.0, 50.0);
        let pt = compute_kinematic_extrapolation(origin, 10.0, 180.0, -2.0, 10.0);
        assert!((pt.0 - 100.0).abs() < 1e-3);
        assert!((pt.1 - 0.0).abs() < 1e-3);
        assert!((pt.2 - 30.0).abs() < 1e-3);

        let pt_west = compute_kinematic_extrapolation(origin, 5.0, 270.0, 1.0, 20.0);
        assert!((pt_west.0 - 0.0).abs() < 1e-3);
        assert!((pt_west.1 - 100.0).abs() < 1e-3);
        assert!((pt_west.2 - 70.0).abs() < 1e-3);
    }

    #[test]
    fn test_cep_radius_min_clamping_and_zero_dt() {
        let cep_clamped = compute_cep_radius_m(0.5, 20.0, 0.0);
        assert!((cep_clamped - 2.0).abs() < 1e-6);

        let cep_nominal = compute_cep_radius_m(10.0, 0.0, 0.0);
        assert!((cep_nominal - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_predict_entity_trajectory_empty_horizons() {
        let pred = predict_entity_trajectory(
            "UAV-EMPTY",
            "GHOST",
            (10.0, 20.0, 30.0),
            0.0,
            0.0,
            false,
            2.0,
            &[],
        );
        assert_eq!(pred.uid, "UAV-EMPTY");
        assert_eq!(pred.callsign, "GHOST");
        assert_eq!(pred.current_enu, (10.0, 20.0, 30.0));
        assert!(pred.points.is_empty());
    }

    #[test]
    fn test_track_predict_request_serde() {
        let req = TrackPredictRequest {
            horizons_sec: Some(vec![5.0, 15.0]),
            stale_threshold_sec: Some(30.0),
        };
        let json = serde_json::to_string(&req).expect("serialize req");
        let de: TrackPredictRequest = serde_json::from_str(&json).expect("deserialize req");
        assert_eq!(de.horizons_sec, Some(vec![5.0, 15.0]));
        assert_eq!(de.stale_threshold_sec, Some(30.0));
    }
}
