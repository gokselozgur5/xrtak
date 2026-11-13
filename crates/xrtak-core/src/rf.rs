//! RF Link Budget, 1st Fresnel Zone & ITU-R Knife-Edge Diffraction Engine.
//!
//! Complies with REQ-RF-01..02 and ICD-19 for tactical datalink margin analysis.

use serde::{Deserialize, Serialize};

/// Standard tactical RF frequency bands and radio link profiles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RfBand {
    /// 900 MHz LoRa / Telemetry / ISR Command link.
    UHF900,
    /// 2.4 GHz MAVLink / C2 / Wi-Fi mesh.
    ISM2400,
    /// 5.8 GHz High-Definition Video / Silvus SC4200 / MPU5 MIMO.
    SHF5800,
}

impl RfBand {
    /// Nominal carrier frequency in Hertz (Hz).
    pub fn frequency_hz(&self) -> f64 {
        match self {
            RfBand::UHF900 => 900_000_000.0,
            RfBand::ISM2400 => 2_400_000_000.0,
            RfBand::SHF5800 => 5_800_000_000.0,
        }
    }

    /// Typical transmitter output power in dBm (1 Watt = +30 dBm).
    pub fn default_tx_power_dbm(&self) -> f64 {
        match self {
            RfBand::UHF900 => 30.0,  // 1W
            RfBand::ISM2400 => 27.0, // 500mW
            RfBand::SHF5800 => 33.0, // 2W MIMO
        }
    }

    /// Default receiver sensitivity threshold in dBm.
    pub fn default_rx_sensitivity_dbm(&self) -> f64 {
        match self {
            RfBand::UHF900 => -110.0,
            RfBand::ISM2400 => -95.0,
            RfBand::SHF5800 => -88.0,
        }
    }
}

/// Evaluated RF Link telemetry and propagation budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfLinkResult {
    pub frequency_hz: f64,
    pub wavelength_m: f64,
    pub distance_m: f64,
    pub fspl_db: f64,
    pub max_fresnel_radius_m: f64,
    pub worst_clearance_m: f64,
    pub diffraction_loss_db: f64,
    pub received_power_dbm: f64,
    pub link_margin_db: f64,
    pub status: String,
}

/// Computes Free-Space Path Loss (FSPL) in decibels.
pub fn compute_fspl_db(distance_m: f64, frequency_hz: f64) -> f64 {
    let d = distance_m.max(1.0);
    let c = 299_792_458.0;
    let lambda = c / frequency_hz;
    20.0 * (4.0 * std::f64::consts::PI * d / lambda).log10()
}

/// Computes 1st Fresnel zone radius F_1 (meters) at distances d1, d2.
pub fn compute_fresnel_radius_m(d1_m: f64, d2_m: f64, frequency_hz: f64) -> f64 {
    let total_d = d1_m + d2_m;
    if total_d <= 1e-3 {
        return 0.0;
    }
    let c = 299_792_458.0;
    let lambda = c / frequency_hz;
    ((lambda * d1_m * d2_m) / total_d).sqrt()
}

/// Computes Knife-Edge Diffraction Loss J(v) in dB using ITU-R P.526-15 approximation.
///
/// v is the dimensionless diffraction parameter.
/// If v <= -0.7, obstacle is outside 0.6 Fresnel clearance (0 dB loss).
pub fn compute_knife_edge_diffraction_db(v: f64) -> f64 {
    if v <= -0.7 {
        0.0
    } else {
        let term = ((v - 0.1).powi(2) + 1.0).sqrt() + (v - 0.1);
        if term <= 0.0 {
            0.0
        } else {
            6.9 + 20.0 * term.log10()
        }
    }
}

/// Computes dimensionless Fresnel-Kirchhoff diffraction parameter v.
///
/// obstacle_penetration_m is obstacle elevation minus LOS ray height at obstacle distance.
pub fn compute_diffraction_v(
    obstacle_penetration_m: f64,
    d1_m: f64,
    d2_m: f64,
    frequency_hz: f64,
) -> f64 {
    let total_d = d1_m + d2_m;
    if total_d <= 1e-3 || d1_m <= 1e-3 || d2_m <= 1e-3 {
        return -1.0;
    }
    let c = 299_792_458.0;
    let lambda = c / frequency_hz;
    let scale = (2.0 * total_d / (lambda * d1_m * d2_m)).sqrt();
    obstacle_penetration_m * scale
}

/// Samples terrain along ray and computes distance, worst clearance, diffraction v, and FSPL.
pub fn evaluate_rf_propagation(
    tx: (f64, f64, f64),
    rx: (f64, f64, f64),
    freq: f64,
    steps: usize,
    provider: &dyn crate::geo::ElevationProvider,
) -> (f64, f64, f64, f64) {
    let dx = rx.0 - tx.0;
    let dy = rx.1 - tx.1;
    let dz = rx.2 - tx.2;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(1.0);
    let n = steps.max(16);
    let mut worst_v = -10.0f64;
    let mut min_clear = f64::MAX;

    for i in 1..n {
        let t = i as f64 / n as f64;
        let ray_alt = tx.2 + t * dz;
        let terrain_alt = provider.sample_relief_m(tx.0 + t * dx, tx.1 + t * dy);
        let clear = ray_alt - terrain_alt;
        if clear < min_clear {
            min_clear = clear;
        }
        let v = compute_diffraction_v(terrain_alt - ray_alt, t * dist, (1.0 - t) * dist, freq);
        if v > worst_v {
            worst_v = v;
        }
    }
    let fspl = compute_fspl_db(dist, freq);
    (dist, min_clear, worst_v, fspl)
}

/// Evaluates an RF link budget across terrain with 1st Fresnel clearance and diffraction.
pub fn evaluate_rf_link(
    tx: (f64, f64, f64),
    rx: (f64, f64, f64),
    band: RfBand,
    steps: usize,
    provider: &dyn crate::geo::ElevationProvider,
) -> RfLinkResult {
    let freq = band.frequency_hz();
    let lambda = 299_792_458.0 / freq;
    let (dist, min_clear, worst_v, fspl) = evaluate_rf_propagation(tx, rx, freq, steps, provider);
    let diffraction = compute_knife_edge_diffraction_db(worst_v);
    let f1_max = compute_fresnel_radius_m(dist / 2.0, dist / 2.0, freq);

    let rx_pwr = band.default_tx_power_dbm() + 4.3 - fspl - diffraction; // 2.15 dBi dipoles
    let margin = rx_pwr - band.default_rx_sensitivity_dbm();

    let status = if margin >= 15.0 {
        "Solid"
    } else if margin >= 5.0 {
        "Marginal"
    } else if margin >= 0.0 {
        "Degraded"
    } else {
        "Severed"
    };

    RfLinkResult {
        frequency_hz: freq,
        wavelength_m: lambda,
        distance_m: dist,
        fspl_db: (fspl * 100.0).round() / 100.0,
        max_fresnel_radius_m: (f1_max * 100.0).round() / 100.0,
        worst_clearance_m: (min_clear * 100.0).round() / 100.0,
        diffraction_loss_db: (diffraction * 100.0).round() / 100.0,
        received_power_dbm: (rx_pwr * 100.0).round() / 100.0,
        link_margin_db: (margin * 100.0).round() / 100.0,
        status: status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fspl_and_fresnel_radius_math() {
        let fspl_2400 = compute_fspl_db(1000.0, 2_400_000_000.0);
        assert!(
            fspl_2400 > 99.0 && fspl_2400 < 101.0,
            "FSPL was {fspl_2400}"
        );
        let r_mid = compute_fresnel_radius_m(500.0, 500.0, 2_400_000_000.0);
        assert!((r_mid - 5.59).abs() < 0.2, "Fresnel radius was {r_mid}");
        let diff_zero = compute_knife_edge_diffraction_db(0.0);
        assert!(
            (diff_zero - 6.0).abs() < 0.5,
            "Diffraction at v=0 was {diff_zero}"
        );
        assert_eq!(compute_knife_edge_diffraction_db(-1.0), 0.0);
    }

    #[test]
    fn test_evaluate_rf_link_knife_edge_diffraction() {
        struct MockRidgeProvider;
        impl crate::geo::ElevationProvider for MockRidgeProvider {
            fn sample_relief_m(&self, x: f64, _y: f64) -> f64 {
                if (x - 500.0).abs() < 50.0 {
                    150.0
                } else {
                    50.0
                }
            }
        }
        let provider = MockRidgeProvider;
        let tx = (0.0, 0.0, 100.0);
        let rx = (1000.0, 0.0, 100.0);
        let res_uhf = evaluate_rf_link(tx, rx, RfBand::UHF900, 32, &provider);
        assert!(
            res_uhf.diffraction_loss_db > 10.0,
            "Diffraction loss was {}",
            res_uhf.diffraction_loss_db
        );
        assert!(res_uhf.worst_clearance_m < 0.0);
    }
}
