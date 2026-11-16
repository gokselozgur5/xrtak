//! Cursor-on-Target (CoT) ping/pong heartbeat keepalive manager.

use serde::{Deserialize, Serialize};

/// Heartbeat status of a TAK peer or server connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartbeatStatus {
    /// Actively receiving pongs within expected interval.
    Connected,
    /// Missed one or more pongs, but within tolerance threshold.
    Degraded,
    /// Timed out or exceeded maximum missed ping count.
    Disconnected,
}

/// Configuration parameters for CoT ping/pong heartbeat keepalive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CotHeartbeatConfig {
    /// Interval between ping heartbeat broadcasts in milliseconds.
    pub interval_ms: u64,
    /// Timeout to wait for a pong response before considering it lost.
    pub timeout_ms: u64,
    /// Maximum consecutive missed pongs before marking connection Disconnected.
    pub max_missed_pongs: u32,
    /// Unique client identifier for CoT event attribution.
    pub client_uid: String,
}

impl Default for CotHeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 15_000,
            timeout_ms: 5_000,
            max_missed_pongs: 3,
            client_uid: "XRTAK-CLIENT".to_string(),
        }
    }
}

/// Record of an active ping transmitted and awaiting pong acknowledgement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingRecord {
    pub seq: u64,
    pub ping_uid: String,
    pub sent_unix_ms: u64,
}

/// Action determined by processing an incoming CoT message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatAction {
    /// Incoming message was a ping; respond with generated pong CoT XML.
    RespondWithPong(String),
    /// Incoming message was a pong acknowledging a pending ping.
    PongAcknowledged { seq: u64, rtt_ms: u64 },
    /// Message is not a heartbeat event or did not match pending pings.
    Ignored,
}

/// State machine managing CoT ping/pong keepalive cycles and latency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CotHeartbeatManager {
    pub config: CotHeartbeatConfig,
    pub(crate) seq: u64,
    pub(crate) last_ping_sent_ms: Option<u64>,
    pub(crate) last_pong_received_ms: Option<u64>,
    pub(crate) last_rtt_ms: Option<u64>,
    pub(crate) smoothed_rtt_ms: f32,
    pub(crate) consecutive_missed: u32,
    pub(crate) total_pings_sent: u64,
    pub(crate) total_pongs_received: u64,
    pub(crate) total_pongs_lost: u64,
    pub(crate) pending_pings: Vec<PingRecord>,
}

impl Default for CotHeartbeatManager {
    fn default() -> Self {
        Self::new(CotHeartbeatConfig::default())
    }
}

impl CotHeartbeatManager {
    /// Create a new heartbeat manager with specified configuration.
    pub fn new(config: CotHeartbeatConfig) -> Self {
        Self {
            config,
            seq: 0,
            last_ping_sent_ms: None,
            last_pong_received_ms: None,
            last_rtt_ms: None,
            smoothed_rtt_ms: 0.0,
            consecutive_missed: 0,
            total_pings_sent: 0,
            total_pongs_received: 0,
            total_pongs_lost: 0,
            pending_pings: Vec::new(),
        }
    }
}

impl CotHeartbeatManager {
    /// Return the current operational status of the heartbeat keepalive.
    pub fn status(&self) -> HeartbeatStatus {
        if self.consecutive_missed >= self.config.max_missed_pongs {
            HeartbeatStatus::Disconnected
        } else if self.consecutive_missed > 0 {
            HeartbeatStatus::Degraded
        } else if self.last_pong_received_ms.is_some() {
            HeartbeatStatus::Connected
        } else {
            HeartbeatStatus::Disconnected
        }
    }

    /// Check if peer link is currently considered active and connected.
    pub fn is_connected(&self) -> bool {
        self.status() == HeartbeatStatus::Connected
    }
}

impl CotHeartbeatManager {
    /// Return the exponentially smoothed round-trip time in milliseconds.
    pub fn smoothed_rtt_ms(&self) -> f32 {
        self.smoothed_rtt_ms
    }

    /// Return the most recent instantaneous round-trip time in milliseconds.
    pub fn last_rtt_ms(&self) -> Option<u64> {
        self.last_rtt_ms
    }

    /// Return packet loss percentage based on total sent and lost pongs.
    pub fn packet_loss_pct(&self) -> f32 {
        if self.total_pings_sent == 0 {
            0.0
        } else {
            (self.total_pongs_lost as f32 / self.total_pings_sent as f32) * 100.0
        }
    }
}

impl CotHeartbeatManager {
    /// Return the count of currently unacknowledged pending pings.
    pub fn pending_count(&self) -> usize {
        self.pending_pings.len()
    }

    /// Return the consecutive number of missed pong timeouts.
    pub fn consecutive_missed(&self) -> u32 {
        self.consecutive_missed
    }
}

/// Build a standard TAK Cursor-on-Target ping event XML (`t-x-c-t`).
pub fn build_ping_cot(client_uid: &str, seq: u64, now_ms: u64) -> String {
    let uid = format!("{client_uid}-ping-{seq}");
    let stale_ms = now_ms.saturating_add(10_000);
    format!(
        r#"<event version="2.0" uid="{uid}" type="t-x-c-t" how="m-g" time="{now_ms}" start="{now_ms}" stale="{stale_ms}"><point lat="0.000000" lon="0.000000" hae="0.0" ce="9999999" le="9999999"/><detail><ping seq="{seq}" client="{client_uid}"/></detail></event>"#
    )
}

/// Build a standard TAK Cursor-on-Target pong response XML (`t-x-c-t-r`).
pub fn build_pong_cot(client_uid: &str, ping_uid: &str, seq: u64, now_ms: u64) -> String {
    let uid = format!("{client_uid}-pong-{seq}");
    let stale_ms = now_ms.saturating_add(10_000);
    format!(
        r#"<event version="2.0" uid="{uid}" type="t-x-c-t-r" how="m-g" time="{now_ms}" start="{now_ms}" stale="{stale_ms}"><point lat="0.000000" lon="0.000000" hae="0.0" ce="9999999" le="9999999"/><detail><pong seq="{seq}" ref_uid="{ping_uid}"/></detail></event>"#
    )
}

impl CotHeartbeatManager {
    /// Check whether elapsed duration requires sending another ping.
    pub fn should_send_ping(&self, now_ms: u64) -> bool {
        match self.last_ping_sent_ms {
            None => true,
            Some(last) => now_ms.saturating_sub(last) >= self.config.interval_ms,
        }
    }

    /// Generate an outgoing ping event and queue record in pending list.
    pub fn generate_ping(&mut self, now_ms: u64) -> (String, u64) {
        self.seq = self.seq.saturating_add(1);
        let seq = self.seq;
        let ping_uid = format!("{}-ping-{seq}", self.config.client_uid);
        let xml = build_ping_cot(&self.config.client_uid, seq, now_ms);
        self.pending_pings.push(PingRecord {
            seq,
            ping_uid,
            sent_unix_ms: now_ms,
        });
        self.last_ping_sent_ms = Some(now_ms);
        self.total_pings_sent = self.total_pings_sent.saturating_add(1);
        (xml, seq)
    }
}

impl CotHeartbeatManager {
    /// Check for unacknowledged pings that exceeded the timeout interval.
    pub fn check_timeouts(&mut self, now_ms: u64) -> u32 {
        let timeout = self.config.timeout_ms;
        let mut expired = 0;
        self.pending_pings.retain(|p| {
            if now_ms.saturating_sub(p.sent_unix_ms) > timeout {
                expired += 1;
                false
            } else {
                true
            }
        });
        if expired > 0 {
            self.consecutive_missed = self.consecutive_missed.saturating_add(expired);
            self.total_pongs_lost = self.total_pongs_lost.saturating_add(expired as u64);
        }
        expired
    }
}

impl CotHeartbeatManager {
    /// Process incoming pong matching ref_uid or sequence number.
    pub fn handle_pong(&mut self, ref_uid: &str, seq: Option<u64>, now_ms: u64) -> Option<u64> {
        let idx = self
            .pending_pings
            .iter()
            .position(|p| p.ping_uid == ref_uid || seq.map_or(false, |s| s == p.seq))?;
        let ping = self.pending_pings.remove(idx);
        let rtt = now_ms.saturating_sub(ping.sent_unix_ms);
        self.last_pong_received_ms = Some(now_ms);
        self.last_rtt_ms = Some(rtt);
        self.smoothed_rtt_ms = if self.smoothed_rtt_ms <= 0.0 {
            rtt as f32
        } else {
            0.8 * self.smoothed_rtt_ms + 0.2 * (rtt as f32)
        };
        self.consecutive_missed = 0;
        self.total_pongs_received = self.total_pongs_received.saturating_add(1);
        Some(rtt)
    }
}

/// Extract the string value of an XML attribute by name.
pub(crate) fn extract_attr<'a>(xml: &'a str, attr_name: &str) -> Option<&'a str> {
    let pattern = format!("{attr_name}=\"");
    let start = xml.find(&pattern)? + pattern.len();
    let end = xml[start..].find('"')? + start;
    Some(&xml[start..end])
}

impl CotHeartbeatManager {
    /// Process an incoming Cursor-on-Target XML packet for heartbeat handling.
    pub fn process_incoming_cot(&mut self, xml: &str, now_ms: u64) -> HeartbeatAction {
        let cot_type = extract_attr(xml, "type").unwrap_or("");
        if cot_type == "t-x-c-t-r" || (cot_type == "t-x-d-d" && xml.contains("pong")) {
            let ref_uid = extract_attr(xml, "ref_uid").unwrap_or("");
            let seq = extract_attr(xml, "seq").and_then(|s| s.parse::<u64>().ok());
            if let Some(rtt) = self.handle_pong(ref_uid, seq, now_ms) {
                return HeartbeatAction::PongAcknowledged {
                    seq: seq.unwrap_or(0),
                    rtt_ms: rtt,
                };
            }
        } else if cot_type == "t-x-c-t" || (cot_type == "t-x-d-d" && xml.contains("ping")) {
            let ping_uid = extract_attr(xml, "uid").unwrap_or("unknown-ping");
            let seq = extract_attr(xml, "seq")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let pong_xml = build_pong_cot(&self.config.client_uid, ping_uid, seq, now_ms);
            return HeartbeatAction::RespondWithPong(pong_xml);
        }
        HeartbeatAction::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_config_and_default_manager() {
        let mgr = CotHeartbeatManager::default();
        assert_eq!(mgr.status(), HeartbeatStatus::Disconnected);
        assert!(!mgr.is_connected());
        assert_eq!(mgr.consecutive_missed(), 0);
        assert_eq!(mgr.pending_count(), 0);
        assert_eq!(mgr.smoothed_rtt_ms(), 0.0);
        assert_eq!(mgr.packet_loss_pct(), 0.0);
    }

    #[test]
    fn test_ping_pong_cot_xml_generation() {
        let ping_xml = build_ping_cot("ALPHA", 42, 1_000_000);
        assert!(ping_xml.contains("type=\"t-x-c-t\""));
        assert!(ping_xml.contains("uid=\"ALPHA-ping-42\""));
        assert!(ping_xml.contains("time=\"1000000\""));

        let pong_xml = build_pong_cot("ALPHA", "BETA-ping-7", 7, 1_000_050);
        assert!(pong_xml.contains("type=\"t-x-c-t-r\""));
        assert!(pong_xml.contains("ref_uid=\"BETA-ping-7\""));
    }

    #[test]
    fn test_generate_ping_and_pending_tracking() {
        let mut mgr = CotHeartbeatManager::default();
        assert!(mgr.should_send_ping(1_000));

        let (xml, seq) = mgr.generate_ping(1_000);
        assert_eq!(seq, 1);
        assert!(xml.contains("uid=\"XRTAK-CLIENT-ping-1\""));
        assert_eq!(mgr.pending_count(), 1);
        assert!(!mgr.should_send_ping(5_000));
        assert!(mgr.should_send_ping(20_000));
    }

    #[test]
    fn test_handle_pong_rtt_and_status() {
        let mut mgr = CotHeartbeatManager::default();
        let (_xml, seq) = mgr.generate_ping(1_000);

        let rtt = mgr.handle_pong("XRTAK-CLIENT-ping-1", Some(seq), 1_045);
        assert_eq!(rtt, Some(45));
        assert_eq!(mgr.last_rtt_ms(), Some(45));
        assert!((mgr.smoothed_rtt_ms() - 45.0).abs() < 1e-4);
        assert_eq!(mgr.status(), HeartbeatStatus::Connected);
        assert!(mgr.is_connected());
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn test_timeout_and_degraded_status() {
        let mut mgr = CotHeartbeatManager::default();
        let (_xml, seq) = mgr.generate_ping(1_000);
        let _ = mgr.handle_pong("XRTAK-CLIENT-ping-1", Some(seq), 1_020);
        assert_eq!(mgr.status(), HeartbeatStatus::Connected);

        let (_xml2, _seq2) = mgr.generate_ping(16_000);
        let expired = mgr.check_timeouts(22_000);
        assert_eq!(expired, 1);
        assert_eq!(mgr.consecutive_missed(), 1);
        assert_eq!(mgr.status(), HeartbeatStatus::Degraded);
    }

    #[test]
    fn test_process_incoming_cot_auto_response() {
        let mut mgr = CotHeartbeatManager::default();
        let ping_xml = r#"<event version="2.0" uid="NODE-ping-9" type="t-x-c-t"><detail><ping seq="9"/></detail></event>"#;
        let action = mgr.process_incoming_cot(ping_xml, 2_000);
        match action {
            HeartbeatAction::RespondWithPong(resp) => {
                assert!(resp.contains("type=\"t-x-c-t-r\""));
                assert!(resp.contains("ref_uid=\"NODE-ping-9\""));
            }
            _ => panic!("Expected RespondWithPong"),
        }
    }

    #[test]
    fn test_process_incoming_cot_pong_acknowledged() {
        let mut mgr = CotHeartbeatManager::default();
        let (_xml, seq) = mgr.generate_ping(5_000);
        let pong_xml = format!(
            r#"<event version="2.0" uid="SERVER-pong-1" type="t-x-c-t-r"><detail><pong seq="{seq}" ref_uid="XRTAK-CLIENT-ping-{seq}"/></detail></event>"#
        );
        let action = mgr.process_incoming_cot(&pong_xml, 5_030);
        match action {
            HeartbeatAction::PongAcknowledged {
                seq: ack_seq,
                rtt_ms,
            } => {
                assert_eq!(ack_seq, seq);
                assert_eq!(rtt_ms, 30);
            }
            _ => panic!("Expected PongAcknowledged"),
        }
    }

    #[test]
    fn test_packet_loss_and_max_missed_disconnect() {
        let mut mgr = CotHeartbeatManager::default();
        let (_xml1, _s1) = mgr.generate_ping(1_000);
        let _ = mgr.check_timeouts(10_000);
        let (_xml2, _s2) = mgr.generate_ping(16_000);
        let _ = mgr.check_timeouts(25_000);
        let (_xml3, _s3) = mgr.generate_ping(31_000);
        let _ = mgr.check_timeouts(40_000);

        assert_eq!(mgr.consecutive_missed(), 3);
        assert_eq!(mgr.status(), HeartbeatStatus::Disconnected);
        assert!(!mgr.is_connected());
        assert!((mgr.packet_loss_pct() - 100.0).abs() < 1e-4);
    }
}
