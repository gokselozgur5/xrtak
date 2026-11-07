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
