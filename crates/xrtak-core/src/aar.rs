use crate::domain::SensorCone;
use crate::TacticalEntity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Single historical keyframe captured by `AarTimeline` (`REQ-AAR-01`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AarFrame {
    pub sim_time_sec: f64,
    pub unix_ms: u64,
    pub entities: Vec<TacticalEntity>,
}

/// Metadata summary of the active 4D AAR ring-buffer (`REQ-AAR-02`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AarSummary {
    pub frame_count: usize,
    pub max_frames: usize,
    pub min_time_sec: f64,
    pub max_time_sec: f64,
}

/// Bounded ring-buffer 4D After-Action Review (AAR) recorder & temporal interpolator (`REQ-AAR-01..03`).
#[derive(Debug, Clone)]
pub struct AarTimeline {
    pub max_frames: usize,
    pub min_capture_interval_sec: f64,
    pub frames: VecDeque<AarFrame>,
}

impl Default for AarTimeline {
    fn default() -> Self {
        Self::new(720, 0.5)
    }
}

impl AarTimeline {
    pub fn new(max_frames: usize, min_capture_interval_sec: f64) -> Self {
        Self {
            max_frames: max_frames.max(8),
            min_capture_interval_sec: min_capture_interval_sec.max(0.05),
            frames: VecDeque::with_capacity(max_frames.max(8)),
        }
    }

    /// Record a keyframe if at least `min_capture_interval_sec` has elapsed since the last frame.
    pub fn maybe_record(
        &mut self,
        sim_time_sec: f64,
        unix_ms: u64,
        entities_map: &HashMap<String, TacticalEntity>,
    ) -> bool {
        if let Some(last) = self.frames.back() {
            if (sim_time_sec - last.sim_time_sec) < self.min_capture_interval_sec {
                return false;
            }
        }
        let mut entities: Vec<TacticalEntity> = entities_map.values().cloned().collect();
        entities.sort_by(|a, b| a.uid.cmp(&b.uid));
        if self.frames.len() >= self.max_frames {
            self.frames.pop_front();
        }
        self.frames.push_back(AarFrame { sim_time_sec, unix_ms, entities });
        true
    }

    pub fn summary(&self) -> AarSummary {
        AarSummary {
            frame_count: self.frames.len(),
            max_frames: self.max_frames,
            min_time_sec: self.frames.front().map(|f| f.sim_time_sec).unwrap_or(0.0),
            max_time_sec: self.frames.back().map(|f| f.sim_time_sec).unwrap_or(0.0),
        }
    }

    /// Reconstruct an interpolated `AarFrame` at arbitrary `target_sim_sec` in $\mathcal{O}(\log N)$ (`REQ-AAR-03`).
    pub fn sample_at(&self, target_sim_sec: f64) -> Option<AarFrame> {
        if self.frames.is_empty() {
            return None;
        }
        if self.frames.len() == 1 || target_sim_sec <= self.frames.front().unwrap().sim_time_sec {
            return self.frames.front().cloned();
        }
        if target_sim_sec >= self.frames.back().unwrap().sim_time_sec {
            return self.frames.back().cloned();
        }

        let upper_idx = self.frames.partition_point(|f| f.sim_time_sec < target_sim_sec);
        let fa = &self.frames[upper_idx.saturating_sub(1)];
        let fb = &self.frames[upper_idx];
        let lambda = ((target_sim_sec - fa.sim_time_sec) / (fb.sim_time_sec - fa.sim_time_sec).max(1e-6)).clamp(0.0, 1.0);
        let fb_map: HashMap<&str, &TacticalEntity> = fb.entities.iter().map(|e| (e.uid.as_str(), e)).collect();

        let entities = fa
            .entities
            .iter()
            .map(|ea| fb_map.get(ea.uid.as_str()).map(|eb| interpolate_entity(ea, eb, lambda)).unwrap_or_else(|| ea.clone()))
            .collect();
        let unix_ms = (fa.unix_ms as f64 + lambda * (fb.unix_ms.saturating_sub(fa.unix_ms) as f64)).round() as u64;
        Some(AarFrame { sim_time_sec: target_sim_sec, unix_ms, entities })
    }
}

/// Shortest-arc angular interpolation in `[0, 360)`.
pub fn lerp_angle_deg(a_deg: f64, b_deg: f64, t: f64) -> f64 {
    let delta = ((b_deg - a_deg + 540.0).rem_euclid(360.0)) - 180.0;
    (a_deg + t * delta).rem_euclid(360.0)
}

fn interpolate_entity(a: &TacticalEntity, b: &TacticalEntity, t: f64) -> TacticalEntity {
    let lerp = |x: f64, y: f64| x + t * (y - x);
    let sensor = match (&a.sensor, &b.sensor) {
        (Some(sa), Some(sb)) => Some(SensorCone {
            azimuth_deg: lerp_angle_deg(sa.azimuth_deg, sb.azimuth_deg, t),
            elevation_deg: lerp(sa.elevation_deg, sb.elevation_deg),
            hfov_deg: lerp(sa.hfov_deg, sb.hfov_deg),
            vfov_deg: lerp(sa.vfov_deg, sb.vfov_deg),
            range_m: lerp(sa.range_m, sb.range_m),
        }),
        (Some(sa), None) => Some(sa.clone()),
        (None, Some(sb)) => Some(sb.clone()),
        (None, None) => None,
    };

    TacticalEntity {
        uid: a.uid.clone(),
        callsign: b.callsign.clone(),
        cot_type: b.cot_type.clone(),
        affiliation: b.affiliation,
        dimension: b.dimension,
        lat: lerp(a.lat, b.lat),
        lon: lerp(a.lon, b.lon),
        hae_m: lerp(a.hae_m, b.hae_m),
        course_deg: lerp_angle_deg(a.course_deg, b.course_deg, t),
        speed_mps: lerp(a.speed_mps, b.speed_mps),
        remarks: b.remarks.clone(),
        sensor,
        video_url: b.video_url.clone(),
        parent_uid: b.parent_uid.clone(),
        east_m: lerp(a.east_m, b.east_m),
        up_m: lerp(a.up_m, b.up_m),
        north_m: lerp(a.north_m, b.north_m),
        updated_unix_ms: b.updated_unix_ms,
    }
}
