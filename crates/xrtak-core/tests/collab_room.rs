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

#[test]
fn prune_stale_respects_ttl_boundary() {
    let mut room = CollabRoomState::new();
    room.peers
        .insert("OP-STALE".into(), make_peer("OP-STALE", "STALE", 10_000));
    room.peers
        .insert("OP-EDGE".into(), make_peer("OP-EDGE", "EDGE", 20_000));
    room.peers
        .insert("OP-FRESH".into(), make_peer("OP-FRESH", "FRESH", 24_000));

    room.prune_stale(25_000, 5_000);

    assert_eq!(room.peers.len(), 2);
    assert!(!room.peers.contains_key("OP-STALE"));
    assert!(room.peers.contains_key("OP-EDGE"));
    assert!(room.peers.contains_key("OP-FRESH"));
}

#[test]
fn sorted_peers_orders_by_operator_id() {
    let mut room = CollabRoomState::new();
    room.peers
        .insert("OP-ZULU".into(), make_peer("OP-ZULU", "ZULU", 1000));
    room.peers
        .insert("OP-ALPHA".into(), make_peer("OP-ALPHA", "ALPHA", 1000));
    room.peers
        .insert("OP-MIKE".into(), make_peer("OP-MIKE", "MIKE", 1000));

    let sorted = room.sorted_peers();
    let ids: Vec<&str> = sorted.iter().map(|p| p.operator_id.as_str()).collect();
    assert_eq!(ids, vec!["OP-ALPHA", "OP-MIKE", "OP-ZULU"]);
}
