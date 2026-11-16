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

    pub fn update(&mut self, obs: &TrackObservation) -> SmoothedTrackState {
        let next_state = match &self.state {
            None => {
                let (vx, vy, speed, course) = match (obs.measured_speed_mps, obs.measured_course_deg) {
                    (Some(spd), Some(crs)) => {
                        let rad = crs.to_radians();
                        (spd * rad.sin(), spd * rad.cos(), spd, wrap_heading_deg(crs))
                    }
                    _ => (0.0, 0.0, 0.0, 0.0),
                };
                SmoothedTrackState {
                    timestamp_sec: obs.timestamp_sec,
                    east_m: obs.east_m,
                    north_m: obs.north_m,
                    up_m: obs.up_m,
                    vel_east_mps: vx,
                    vel_north_mps: vy,
                    vel_up_mps: 0.0,
                    speed_mps: speed,
                    course_deg: course,
                }
            }

            Some(prev) => {
                let dt = (obs.timestamp_sec - prev.timestamp_sec).max(1e-4);
                let raw_vx = (obs.east_m - prev.east_m) / dt;
                let raw_vy = (obs.north_m - prev.north_m) / dt;
                let raw_vz = (obs.up_m - prev.up_m) / dt;

                let (target_vx, target_vy) = match (obs.measured_speed_mps, obs.measured_course_deg) {
                    (Some(spd), Some(crs)) => {
                        let rad = crs.to_radians();
                        (0.5 * raw_vx + 0.5 * spd * rad.sin(), 0.5 * raw_vy + 0.5 * spd * rad.cos())
                    }
                    _ => (raw_vx, raw_vy),
                };

                let mut s_vx = smooth_scalar(prev.vel_east_mps, target_vx, self.config.alpha_vel);
                let mut s_vy = smooth_scalar(prev.vel_north_mps, target_vy, self.config.alpha_vel);
                let s_vz = smooth_scalar(prev.vel_up_mps, raw_vz, self.config.alpha_vel);

                let mut speed = (s_vx.powi(2) + s_vy.powi(2)).sqrt();
                if let Some(clamp) = self.config.max_speed_clamp_mps {
                    if speed > clamp && speed > 1e-6 {
                        let ratio = clamp / speed;
                        s_vx *= ratio;
                        s_vy *= ratio;
                        speed = clamp;
                    }
                }

                let course = if speed >= self.config.min_velocity_threshold_mps {
                    let raw_course = wrap_heading_deg(s_vx.atan2(s_vy).to_degrees());
                    smooth_heading_deg(prev.course_deg, raw_course, self.config.alpha_heading)
                } else {
                    prev.course_deg
                };

                let s_east = smooth_scalar(prev.east_m + s_vx * dt, obs.east_m, self.config.alpha_pos);
                let s_north = smooth_scalar(prev.north_m + s_vy * dt, obs.north_m, self.config.alpha_pos);
                let s_up = smooth_scalar(prev.up_m + s_vz * dt, obs.up_m, self.config.alpha_pos);
