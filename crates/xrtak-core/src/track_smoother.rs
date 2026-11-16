//! Velocity and heading filter smoother for tactical entity tracks.

use serde::{Deserialize, Serialize};

/// Normalizes heading in degrees to [0.0, 360.0).
pub fn wrap_heading_deg(deg: f64) -> f64 {
    (deg % 360.0 + 360.0) % 360.0
}

/// Computes signed shortest angular difference from -> to in [-180.0, 180.0).
pub fn heading_delta_deg(from_deg: f64, to_deg: f64) -> f64 {
    let mut diff = (to_deg - from_deg) % 360.0;
    if diff >= 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    diff
}

/// Smooths heading using shortest angular path across 0/360 boundary.
pub fn smooth_heading_deg(prev_deg: f64, new_deg: f64, alpha: f64) -> f64 {
    let a = alpha.clamp(0.0, 1.0);
    let delta = heading_delta_deg(prev_deg, new_deg);
    wrap_heading_deg(prev_deg + a * delta)
}

/// Computes scalar exponential moving average with clamping.
pub fn smooth_scalar(prev: f64, new_val: f64, alpha: f64) -> f64 {
    let a = alpha.clamp(0.0, 1.0);
    prev + a * (new_val - prev)
}

/// Configuration parameters for track velocity and heading filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSmootherConfig {
    pub alpha_pos: f64,
    pub alpha_vel: f64,
    pub alpha_heading: f64,
    pub min_velocity_threshold_mps: f64,
    pub max_speed_clamp_mps: Option<f64>,
}

impl Default for TrackSmootherConfig {
    fn default() -> Self {
        Self {
            alpha_pos: 0.35,
            alpha_vel: 0.25,
            alpha_heading: 0.20,
            min_velocity_threshold_mps: 0.5,
            max_speed_clamp_mps: Some(150.0),
        }
    }
}

/// Raw position and kinematic observation for track smoothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackObservation {
    pub timestamp_sec: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
    pub measured_speed_mps: Option<f64>,
    pub measured_course_deg: Option<f64>,
}

impl TrackObservation {
    pub fn new(timestamp_sec: f64, east_m: f64, north_m: f64, up_m: f64) -> Self {
        Self {
            timestamp_sec,
            east_m,
            north_m,
            up_m,
            measured_speed_mps: None,
            measured_course_deg: None,
        }
    }
}

impl TrackObservation {
    pub fn with_kinematics(
        timestamp_sec: f64,
        east_m: f64,
        north_m: f64,
        up_m: f64,
        speed_mps: f64,
        course_deg: f64,
    ) -> Self {
        Self {
            timestamp_sec,
            east_m,
            north_m,
            up_m,
            measured_speed_mps: Some(speed_mps),
            measured_course_deg: Some(wrap_heading_deg(course_deg)),
        }
    }
}

/// Filtered kinematic state with smoothed velocity vector and course.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmoothedTrackState {
    pub timestamp_sec: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
    pub vel_east_mps: f64,
    pub vel_north_mps: f64,
    pub vel_up_mps: f64,
    pub speed_mps: f64,
    pub course_deg: f64,
}

/// Stateful velocity and heading filter smoother.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSmoother {
    pub config: TrackSmootherConfig,
    state: Option<SmoothedTrackState>,
}

impl Default for TrackSmoother {
    fn default() -> Self {
        Self::new(TrackSmootherConfig::default())
    }
}

impl TrackSmoother {
    pub fn new(config: TrackSmootherConfig) -> Self {
        Self {
            config,
            state: None,
        }
    }

    pub fn current_state(&self) -> Option<&SmoothedTrackState> {
        self.state.as_ref()
    }

    pub fn reset(&mut self) {
        self.state = None;
    }
