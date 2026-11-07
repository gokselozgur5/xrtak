use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorPeer {
    pub operator_id: String,
    pub callsign: String,
    /// e.g. "QUEST3-AR", "DESKTOP-C2"
    pub device_mode: String,
    /// Operator head/camera position in table-relative ENU meters `(east_m, up_m, north_m)`
    pub head_enu: (f64, f64, f64),
    /// Active 3D laser pointer intersection on terrain `(east_m, up_m, north_m)`
    pub laser_target_enu: Option<(f64, f64, f64)>,
    /// True while Push-to-Talk voice transmission is active
    pub ptt_active: bool,
    pub color_hex: String,
    pub updated_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SharedTablePose {
    pub scale: f64,
    pub yaw_deg: f64,
    pub updated_unix_ms: u64,
}

impl Default for SharedTablePose {
    fn default() -> Self {
        Self {
            scale: 1.0,
            yaw_deg: 0.0,
            updated_unix_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollabRoomState {
    pub peers: HashMap<String, OperatorPeer>,
    pub table_pose: SharedTablePose,
}

impl CollabRoomState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Upserts an operator peer and optionally updates the shared table pose (`REQ-RTC-01`).
    pub fn upsert_peer(
        &mut self,
        peer: OperatorPeer,
        shared_scale: Option<f64>,
        shared_yaw_deg: Option<f64>,
    ) {
        let now_ms = peer.updated_unix_ms;
        if let Some(s) = shared_scale {
            self.table_pose.scale = s.clamp(0.25, 4.0);
            self.table_pose.updated_unix_ms = now_ms;
        }
        if let Some(y) = shared_yaw_deg {
            self.table_pose.yaw_deg = y.rem_euclid(360.0);
            self.table_pose.updated_unix_ms = now_ms;
        }
        self.peers.insert(peer.operator_id.clone(), peer);
        self.prune_stale(now_ms, 15_000);
    }

    pub fn prune_stale(&mut self, now_unix_ms: u64, ttl_ms: u64) {
        self.peers
            .retain(|_, p| now_unix_ms.saturating_sub(p.updated_unix_ms) <= ttl_ms);
    }

    pub fn sorted_peers(&self) -> Vec<OperatorPeer> {
        let mut list: Vec<OperatorPeer> = self.peers.values().cloned().collect();
        list.sort_by(|a, b| a.operator_id.cmp(&b.operator_id));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_operator_presence_and_laser_sync() {
        let mut room = CollabRoomState::new();
        room.upsert_peer(
            OperatorPeer {
                operator_id: "OP-QUEST-01".to_string(),
                callsign: "ALPHA-XR".to_string(),
                device_mode: "QUEST3-AR".to_string(),
                head_enu: (0.0, 420.0, -350.0),
                laser_target_enu: Some((125.0, 45.0, 210.0)),
                ptt_active: true,
                color_hex: "#00ff88".to_string(),
                updated_unix_ms: 10_000,
            },
            Some(1.45),
            Some(35.0),
        );

        room.upsert_peer(
            OperatorPeer {
                operator_id: "OP-DESK-02".to_string(),
                callsign: "BRAVO-C2".to_string(),
                device_mode: "DESKTOP-C2".to_string(),
                head_enu: (-180.0, 510.0, 120.0),
                laser_target_enu: None,
                ptt_active: false,
                color_hex: "#38bdf8".to_string(),
                updated_unix_ms: 12_000,
            },
            None,
            None,
        );

        assert_eq!(room.sorted_peers().len(), 2);
        assert!((room.table_pose.scale - 1.45).abs() < 1e-6);
        assert!((room.table_pose.yaw_deg - 35.0).abs() < 1e-6);

        // Advance past 15s TTL for OP-QUEST-01 (10_000 + 16_000 = 26_000)
        room.prune_stale(26_000, 15_000);
        let remaining = room.sorted_peers();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].operator_id, "OP-DESK-02");
    }
}
