use xrtak_core::collab::{CollabRoomState, OperatorPeer};

fn make_peer(id: &str, callsign: &str, t_ms: u64) -> OperatorPeer {
    OperatorPeer {
        operator_id: id.into(),
        callsign: callsign.into(),
        device_mode: "QUEST3".into(),
        head_enu: (0.0, 0.0, 0.0),
        laser_target_enu: None,
        ptt_active: false,
        color_hex: "#fff".into(),
        updated_unix_ms: t_ms,
    }
}

#[test]
fn upsert_peer_updates_in_place() {
    let mut room = CollabRoomState::new();
    room.upsert_peer(make_peer("OP-1", "ALPHA", 1000), None, None);
    assert_eq!(room.peers.len(), 1);

    room.upsert_peer(make_peer("OP-1", "BRAVO", 2000), None, None);
    assert_eq!(room.peers.len(), 1);
    assert_eq!(room.peers.get("OP-1").unwrap().callsign, "BRAVO");
}
