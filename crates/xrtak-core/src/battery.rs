// crates/xrtak-core/src/battery.rs
//! UAV Wind-Corrected Bingo Battery & Asymmetric RTH Polygon Engine (ICD-26, REQ-BAT-01..02).

use serde::{Deserialize, Serialize};

/// Ambient wind vector in local ENU frame.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindVector {
    /// Wind blowing towards East (m/s)
    pub east_mps: f64,
    /// Wind blowing towards North (m/s)
    pub north_mps: f64,
}

impl Default for WindVector {
    fn default() -> Self {
        Self { east_mps: 0.0, north_mps: 0.0 }
    }
}

/// UAV Battery state and electrical parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UavBatteryState {
    pub callsign: String,
    pub battery_remaining_pct: f64,
    pub battery_capacity_wh: f64,
    pub current_power_draw_w: f64,
    pub reserve_margin_pct: f64,
    pub cruise_airspeed_mps: f64,
    pub current_east_m: f64,
    pub current_north_m: f64,
    pub current_up_m: f64,
    pub home_east_m: f64,
    pub home_north_m: f64,
    pub home_up_m: f64,
}

/// 2D Point on RTH footprint perimeter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RthPolygonVertex {
    pub bearing_deg: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub max_radius_m: f64,
}

/// Comprehensive UAV battery endurance assessment and asymmetric RTH polygon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UavEnduranceSolution {
    pub callsign: String,
    pub total_endurance_sec: f64,
    pub remaining_flight_sec: f64,
    pub return_home_time_sec: f64,
    pub bingo_battery_pct: f64,
    pub is_bingo_critical: bool,
    pub distance_to_home_m: f64,
    pub rth_polygon: Vec<RthPolygonVertex>,
}

/// Compute wind-corrected bingo battery threshold and asymmetric return polygon.
pub fn compute_uav_endurance(
    state: &UavBatteryState,
    wind: &WindVector,
    polygon_radial_steps: usize,
) -> UavEnduranceSolution {
    let power_w = state.current_power_draw_w.max(10.0);
    let rem_wh = (state.battery_remaining_pct / 100.0) * state.battery_capacity_wh;
    let total_rem_sec = (rem_wh / power_w) * 3600.0;

    let res_wh = (state.reserve_margin_pct / 100.0) * state.battery_capacity_wh;
    let usable_sec = (((rem_wh - res_wh).max(0.0)) / power_w) * 3600.0;

    let dx_home = state.home_east_m - state.current_east_m;
    let dy_home = state.home_north_m - state.current_north_m;
    let dist_to_home = (dx_home.powi(2) + dy_home.powi(2)).sqrt();

    let bearing_to_home_rad = dy_home.atan2(dx_home);
    let headwind_comp = wind.east_mps * bearing_to_home_rad.cos() + wind.north_mps * bearing_to_home_rad.sin();
    let ground_speed_home = (state.cruise_airspeed_mps + headwind_comp).max(1.0);
    let rth_time_sec = dist_to_home / ground_speed_home;

    let energy_home_wh = (rth_time_sec / 3600.0) * power_w;
    let bingo_wh = energy_home_wh + res_wh;
    let bingo_pct = (bingo_wh / state.battery_capacity_wh.max(1.0)) * 100.0;
    let is_bingo_critical = state.battery_remaining_pct <= bingo_pct;

    let steps = polygon_radial_steps.max(16);
    let mut polygon = Vec::with_capacity(steps);

    for i in 0..steps {
        let frac = i as f64 / steps as f64;
        let az_rad = frac * 2.0 * std::f64::consts::PI;
        let bearing_deg = az_rad.to_degrees();

        let headwind = wind.east_mps * az_rad.cos() + wind.north_mps * az_rad.sin();
        let ground_speed = (state.cruise_airspeed_mps + headwind).max(1.0);
        let max_r = (usable_sec * ground_speed * 0.5).max(10.0);

        polygon.push(RthPolygonVertex {
            bearing_deg: (bearing_deg * 10.0).round() / 10.0,
            east_m: (state.current_east_m + max_r * az_rad.cos() * 10.0).round() / 10.0,
            north_m: (state.current_north_m + max_r * az_rad.sin() * 10.0).round() / 10.0,
            max_radius_m: (max_r * 10.0).round() / 10.0,
        });
    }

    UavEnduranceSolution {
        callsign: state.callsign.clone(),
        total_endurance_sec: (total_rem_sec * 10.0).round() / 10.0,
        remaining_flight_sec: (usable_sec * 10.0).round() / 10.0,
        return_home_time_sec: (rth_time_sec * 10.0).round() / 10.0,
        bingo_battery_pct: (bingo_pct * 10.0).round() / 10.0,
        is_bingo_critical,
        distance_to_home_m: (dist_to_home * 10.0).round() / 10.0,
        rth_polygon: polygon,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_compute_uav_endurance_and_bingo() {
        let state = UavBatteryState {
            callsign: "VALKYRIE-1".to_string(),
            battery_remaining_pct: 65.0,
            battery_capacity_wh: 120.0,
            current_power_draw_w: 180.0,
            reserve_margin_pct: 15.0,
            cruise_airspeed_mps: 18.0,
            current_east_m: 1200.0,
            current_north_m: 800.0,
            current_up_m: 150.0,
            home_east_m: 0.0,
            home_north_m: 0.0,
            home_up_m: 20.0,
        };
        let wind = WindVector {
            east_mps: -5.0,
            north_mps: 2.0,
        };
        let sol = compute_uav_endurance(&state, &wind, 24);
        assert!(sol.remaining_flight_sec > 0.0);
        assert!(sol.bingo_battery_pct > 15.0);
        assert!(!sol.is_bingo_critical);
        assert_eq!(sol.rth_polygon.len(), 24);
    }

    #[test]
    fn test_bingo_critical_state() {
        let state = UavBatteryState {
            callsign: "VALKYRIE-2".to_string(),
            battery_remaining_pct: 18.0,
            battery_capacity_wh: 100.0,
            current_power_draw_w: 200.0,
            reserve_margin_pct: 15.0,
            cruise_airspeed_mps: 15.0,
            current_east_m: 4000.0,
            current_north_m: 3000.0,
            current_up_m: 200.0,
            home_east_m: 0.0,
            home_north_m: 0.0,
            home_up_m: 10.0,
        };
        let wind = WindVector {
            east_mps: 10.0,
            north_mps: 5.0,
        };
        let sol = compute_uav_endurance(&state, &wind, 16);
        assert!(sol.is_bingo_critical);
    }
}
