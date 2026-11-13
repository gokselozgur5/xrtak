// crates/xrtak-core/src/fires.rs
//! 3D Ballistic Trajectory, Maximum Ordinate & Crest Clearance Engine (ICD-25, REQ-FIR-01..02).

use crate::geo::ElevationProvider;
use serde::{Deserialize, Serialize};

/// Fire support munition / trajectory profile.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MunitionType {
    /// 155mm Howitzer / Artillery shell (high angle or low angle)
    Artillery155mm,
    /// 120mm Heavy Mortar (high angle plunging)
    Mortar120mm,
    /// Loitering Munition / Kamikaze Drone (glide profile)
    LoiteringMunition,
    /// Direct Rocket / Guided Rocket (MLRS)
    TacticalRocket,
}

impl MunitionType {
    pub fn nominal_velocity_mps(&self) -> f64 {
        match self {
            Self::Artillery155mm => 580.0,
            Self::Mortar120mm => 240.0,
            Self::LoiteringMunition => 65.0,
            Self::TacticalRocket => 820.0,
        }
    }
}

/// Single sample along 3D ballistic trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint3d {
    pub time_sec: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
    pub terrain_elev_m: f64,
    pub clearance_m: f64,
}

/// Ballistic fire mission solution and terrain crest analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallisticSolution {
    pub munition: MunitionType,
    pub ground_range_m: f64,
    pub time_of_flight_sec: f64,
    pub max_ordinate_m: f64,
    pub min_crest_clearance_m: f64,
    pub is_crest_cleared: bool,
    pub trajectory: Vec<TrajectoryPoint3d>,
}

/// Compute 3D ballistic trajectory arc and verify crest clearance against DEM.
pub fn compute_ballistic_trajectory(
    origin_enu: (f64, f64, f64),
    target_enu: (f64, f64, f64),
    munition: MunitionType,
    elevation_angle_deg: Option<f64>,
    elevation_provider: &dyn ElevationProvider,
    samples: usize,
) -> BallisticSolution {
    let dx = target_enu.0 - origin_enu.0;
    let dy = target_enu.1 - origin_enu.1;
    let dz = target_enu.2 - origin_enu.2;
    let ground_range = (dx.powi(2) + dy.powi(2)).sqrt().max(1.0);

    let g = 9.80665;
    let v0 = munition.nominal_velocity_mps();

    let theta_rad = if let Some(deg) = elevation_angle_deg {
        deg.to_radians()
    } else {
        match munition {
            MunitionType::Mortar120mm => 65.0_f64.to_radians(),
            MunitionType::LoiteringMunition => 18.0_f64.to_radians(),
            _ => 35.0_f64.to_radians(),
        }
    };

    let v_h = v0 * theta_rad.cos();
    let v_z = v0 * theta_rad.sin();
    let tof = (ground_range / v_h.max(1.0)).max(0.5);

    let n_pts = samples.max(16);
    let mut trajectory = Vec::with_capacity(n_pts);
    let mut max_ord: f64 = origin_enu.2;
    let mut min_clearance: f64 = f64::MAX;

    for i in 0..=n_pts {
        let frac = i as f64 / n_pts as f64;
        let t = frac * tof;
        let curr_e = origin_enu.0 + frac * dx;
        let curr_n = origin_enu.1 + frac * dy;
        let z_ballistic = origin_enu.2 + (v_z * t) - (0.5 * g * t.powi(2));
        let curr_u = z_ballistic + frac * (dz - ((v_z * tof) - (0.5 * g * tof.powi(2))));

        if curr_u > max_ord {
            max_ord = curr_u;
        }

        let terrain_z = elevation_provider.sample_relief_m(curr_e, curr_n);
        let clearance = curr_u - terrain_z;
        if i > 0 && i < n_pts && clearance < min_clearance {
            min_clearance = clearance;
        }

        trajectory.push(TrajectoryPoint3d {
            time_sec: (t * 100.0).round() / 100.0,
            east_m: (curr_e * 10.0).round() / 10.0,
            north_m: (curr_n * 10.0).round() / 10.0,
            up_m: (curr_u * 10.0).round() / 10.0,
            terrain_elev_m: (terrain_z * 10.0).round() / 10.0,
            clearance_m: (clearance * 10.0).round() / 10.0,
        });
    }

    if min_clearance == f64::MAX {
        min_clearance = 0.0;
    }

    let is_crest_cleared = min_clearance >= 0.0;

    BallisticSolution {
        munition,
        ground_range_m: (ground_range * 10.0).round() / 10.0,
        time_of_flight_sec: (tof * 10.0).round() / 10.0,
        max_ordinate_m: (max_ord * 10.0).round() / 10.0,
        min_crest_clearance_m: (min_clearance * 10.0).round() / 10.0,
        is_crest_cleared,
        trajectory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::ProceduralReliefModel;

    #[test]
    fn test_compute_ballistic_trajectory_mortar_plunging() {
        let model = ProceduralReliefModel;
        let sol = compute_ballistic_trajectory(
            (0.0, 0.0, 50.0),
            (1500.0, 1500.0, 80.0),
            MunitionType::Mortar120mm,
            Some(60.0),
            &model,
            24,
        );
        assert!(sol.ground_range_m > 2000.0);
        assert!(sol.max_ordinate_m > 200.0);
        assert!(sol.time_of_flight_sec > 10.0);
        assert_eq!(sol.trajectory.len(), 25);
    }

    #[test]
    fn test_compute_ballistic_trajectory_artillery_crest_clearance() {
        let model = ProceduralReliefModel;
        let sol = compute_ballistic_trajectory(
            (-1000.0, -1000.0, 100.0),
            (1000.0, 1000.0, 120.0),
            MunitionType::Artillery155mm,
            Some(40.0),
            &model,
            32,
        );
        assert!(sol.is_crest_cleared);
        assert!(sol.max_ordinate_m > sol.min_crest_clearance_m);
    }
}
