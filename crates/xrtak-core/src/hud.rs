use serde::{Deserialize, Serialize};

/// First-person Passthrough HUD cue relative to the operator's ENU position (SRP).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HudCue {
    pub bearing_deg: f64,
    pub slant_range_m: f64,
    pub elevation_deg: f64,
}

/// Computes azimuth bearing (0° = North, 90° = East), slant range, and elevation angle.
pub fn compute_hud_cue(observer_enu: (f64, f64, f64), target_enu: (f64, f64, f64)) -> HudCue {
    let dx = target_enu.0 - observer_enu.0;
    let dy = target_enu.1 - observer_enu.1;
    let dz = target_enu.2 - observer_enu.2;

    let ground_range = (dx * dx + dz * dz).sqrt();
    let slant_range_m = (ground_range * ground_range + dy * dy).sqrt();
    let bearing_deg = dx.atan2(dz).to_degrees().rem_euclid(360.0);
    let elevation_deg = dy.atan2(ground_range.max(1e-6)).to_degrees();

    HudCue {
        bearing_deg,
        slant_range_m,
        elevation_deg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_cue_north_east_bearing_and_range() {
        let cue = compute_hud_cue((0.0, 0.0, 0.0), (100.0, 100.0, 0.0));
        assert!((cue.bearing_deg - 90.0).abs() < 1e-4);
        assert!((cue.elevation_deg - 45.0).abs() < 1e-4);
    }
}
