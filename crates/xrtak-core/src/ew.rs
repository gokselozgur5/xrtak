// crates/xrtak-core/src/ew.rs
//! Electronic Warfare (EW) Jammer-to-Signal & Burn-Through Analysis Engine (ICD-23, REQ-EW-01..02).

use serde::{Deserialize, Serialize};

/// Target link / EW frequency band.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EwTargetBand {
    /// GPS L1 C/A (1575.42 MHz)
    GnssL1,
    /// Tactical C2 / MAVLink (915.0 MHz)
    TacticalC2,
    /// High-throughput video downlink (2400.0 MHz)
    VideoDownlink,
    /// X-band radar (9500.0 MHz)
    RadarXBand,
}

impl EwTargetBand {
    /// Carrier center frequency in MHz.
    pub fn frequency_mhz(&self) -> f64 {
        match self {
            Self::GnssL1 => 1575.42,
            Self::TacticalC2 => 915.0,
            Self::VideoDownlink => 2400.0,
            Self::RadarXBand => 9500.0,
        }
    }
}

/// Jammer parameters and operational geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JammerProfile {
    pub callsign: String,
    pub transmitter_power_w: f64,
    pub antenna_gain_dbi: f64,
    pub bandwidth_mhz: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
}

/// Target receiver / transceiver parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLinkProfile {
    pub callsign: String,
    pub transmitter_power_w: f64,
    pub tx_antenna_gain_dbi: f64,
    pub rx_antenna_gain_dbi: f64,
    pub signal_bandwidth_mhz: f64,
    pub receiver_sensitivity_dbm: f64,
    pub tx_east_m: f64,
    pub tx_north_m: f64,
    pub tx_up_m: f64,
    pub rx_east_m: f64,
    pub rx_north_m: f64,
    pub rx_up_m: f64,
}

/// Convert Watts to dBm.
pub fn watts_to_dbm(watts: f64) -> f64 {
    if watts <= 0.0 {
        -150.0
    } else {
        10.0 * (watts * 1000.0).log10()
    }
}

/// Free-space path loss in dB: FSPL = 20*log10(d_km) + 20*log10(f_mhz) + 32.44
pub fn compute_fspl_db(distance_m: f64, frequency_mhz: f64) -> f64 {
    let d_km = (distance_m / 1000.0).max(0.001);
    let f_mhz = frequency_mhz.max(1.0);
    20.0 * d_km.log10() + 20.0 * f_mhz.log10() + 32.44
}

/// Evaluation result of Jammer-to-Signal ratio and link denial status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JammingAssessment {
    pub jammer_callsign: String,
    pub target_callsign: String,
    pub frequency_mhz: f64,
    pub distance_to_jammer_m: f64,
    pub distance_to_transmitter_m: f64,
    pub received_signal_power_dbm: f64,
    pub received_jamming_power_dbm: f64,
    pub jammer_to_signal_ratio_db: f64,
    pub is_denied: bool,
    pub suppression_radius_m: f64,
}

/// Compute Jammer-to-Signal ratio and suppression geometry.
pub fn evaluate_link_jamming(
    jammer: &JammerProfile,
    link: &TargetLinkProfile,
    band: EwTargetBand,
    jamming_margin_threshold_db: f64,
) -> JammingAssessment {
    let freq = band.frequency_mhz();
    let dx_sig = link.rx_east_m - link.tx_east_m;
    let dy_sig = link.rx_north_m - link.tx_north_m;
    let dz_sig = link.rx_up_m - link.tx_up_m;
    let dist_sig = (dx_sig.powi(2) + dy_sig.powi(2) + dz_sig.powi(2)).sqrt().max(1.0);

    let dx_jam = link.rx_east_m - jammer.east_m;
    let dy_jam = link.rx_north_m - jammer.north_m;
    let dz_jam = link.rx_up_m - jammer.up_m;
    let dist_jam = (dx_jam.powi(2) + dy_jam.powi(2) + dz_jam.powi(2)).sqrt().max(1.0);

    let fspl_sig = compute_fspl_db(dist_sig, freq);
    let fspl_jam = compute_fspl_db(dist_jam, freq);

    let p_tx_dbm = watts_to_dbm(link.transmitter_power_w);
    let p_rx_sig = p_tx_dbm + link.tx_antenna_gain_dbi + link.rx_antenna_gain_dbi - fspl_sig;

    let p_jam_dbm = watts_to_dbm(jammer.transmitter_power_w);
    let bw_correction_db = 10.0 * (link.signal_bandwidth_mhz / jammer.bandwidth_mhz.max(0.1)).min(1.0).log10();
    let p_rx_jam = p_jam_dbm + jammer.antenna_gain_dbi + link.rx_antenna_gain_dbi - fspl_jam + bw_correction_db;

    let js_ratio = p_rx_jam - p_rx_sig;
    let is_denied = js_ratio >= jamming_margin_threshold_db || p_rx_sig < link.receiver_sensitivity_dbm;

    let delta_db = (p_jam_dbm + jammer.antenna_gain_dbi + bw_correction_db) - (p_rx_sig + jamming_margin_threshold_db - link.rx_antenna_gain_dbi);
    let supp_km = 10.0_f64.powf((delta_db - 32.44 - 20.0 * freq.log10()) / 20.0);
    let suppression_radius_m = (supp_km * 1000.0).max(10.0).min(100_000.0);

    JammingAssessment {
        jammer_callsign: jammer.callsign.clone(),
        target_callsign: link.callsign.clone(),
        frequency_mhz: freq,
        distance_to_jammer_m: dist_jam,
        distance_to_transmitter_m: dist_sig,
        received_signal_power_dbm: (p_rx_sig * 100.0).round() / 100.0,
        received_jamming_power_dbm: (p_rx_jam * 100.0).round() / 100.0,
        jammer_to_signal_ratio_db: (js_ratio * 100.0).round() / 100.0,
        is_denied,
        suppression_radius_m: (suppression_radius_m * 10.0).round() / 10.0,
    }
}

/// Burn-through evaluation against standoff radar noise jamming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnThroughAssessment {
    pub radar_callsign: String,
    pub jammer_callsign: String,
    pub burn_through_range_m: f64,
    pub target_rcs_dbsm: f64,
    pub is_burned_through: bool,
}

/// Compute radar burn-through range R_BT where target echo power >= jammer noise.
pub fn compute_burn_through_range(
    radar_callsign: &str,
    p_radar_w: f64,
    g_radar_dbi: f64,
    target_rcs_m2: f64,
    jammer_callsign: &str,
    p_jammer_w: f64,
    g_jammer_dbi: f64,
    distance_radar_to_jammer_m: f64,
    current_target_range_m: f64,
) -> BurnThroughAssessment {
    let g_rad_linear = 10.0_f64.powf(g_radar_dbi / 10.0);
    let g_jam_linear = 10.0_f64.powf(g_jammer_dbi / 10.0);
    let r_j = distance_radar_to_jammer_m.max(100.0);

    let numerator = p_radar_w * g_rad_linear * target_rcs_m2.max(0.01) * (r_j.powi(2));
    let denominator = (4.0 * std::f64::consts::PI) * (p_jammer_w * g_jam_linear).max(0.001);
    let r_bt = (numerator / denominator).powf(0.5).max(10.0);

    BurnThroughAssessment {
        radar_callsign: radar_callsign.to_string(),
        jammer_callsign: jammer_callsign.to_string(),
        burn_through_range_m: (r_bt * 10.0).round() / 10.0,
        target_rcs_dbsm: (10.0 * target_rcs_m2.max(0.001).log10() * 100.0).round() / 100.0,
        is_burned_through: current_target_range_m <= r_bt,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_link_jamming_denial_and_js_ratio() {
        let jammer = JammerProfile {
            callsign: "KRASUKHA-4".to_string(),
            transmitter_power_w: 100.0,
            antenna_gain_dbi: 15.0,
            bandwidth_mhz: 20.0,
            east_m: 1000.0,
            north_m: 1000.0,
            up_m: 100.0,
        };
        let link = TargetLinkProfile {
            callsign: "FALCON-1".to_string(),
            transmitter_power_w: 1.0,
            tx_antenna_gain_dbi: 2.0,
            rx_antenna_gain_dbi: 0.0,
            signal_bandwidth_mhz: 2.0,
            receiver_sensitivity_dbm: -100.0,
            tx_east_m: 0.0,
            tx_north_m: 0.0,
            tx_up_m: 10.0,
            rx_east_m: 800.0,
            rx_north_m: 800.0,
            rx_up_m: 150.0,
        };
        let assessment = evaluate_link_jamming(&jammer, &link, EwTargetBand::TacticalC2, 6.0);
        assert!(assessment.jammer_to_signal_ratio_db > 0.0);
        assert!(assessment.is_denied);
        assert!(assessment.suppression_radius_m > 100.0);
    }

    #[test]
    fn test_radar_burn_through_range() {
        let bt = compute_burn_through_range(
            "PATRIOT-RADAR",
            50_000.0,
            30.0,
            2.0,
            "POD-JAMMER",
            500.0,
            10.0,
            50_000.0,
            12_000.0,
        );
        assert!(bt.burn_through_range_m > 1000.0);
        assert!(bt.target_rcs_dbsm > 0.0);
    }
}
