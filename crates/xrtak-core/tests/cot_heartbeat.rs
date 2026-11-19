use xrtak_core::{build_ping_cot, build_pong_cot, CotHeartbeatManager, HeartbeatAction};

#[test]
fn tak_server_pong_acknowledges_pending_ping() {
    let mut mgr = CotHeartbeatManager::default();
    let _ = mgr.generate_ping(1_000);
    let pong = r#"<event version="2.0" uid="takPong" type="t-x-c-t-r" how="h-g-i-g-o" time="2026-11-18T10:00:00Z" start="2026-11-18T10:00:00Z" stale="2026-11-18T10:00:20Z"><point lat="0.0" lon="0.0" hae="0.0" ce="9999999" le="9999999"/></event>"#;
    let action = mgr.process_incoming_cot(pong, 1_040);
    assert!(
        matches!(action, HeartbeatAction::PongAcknowledged { rtt_ms: 40, .. }),
        "pong was {action:?}"
    );
    assert!(mgr.is_connected());
}

#[test]
fn heartbeat_events_use_iso8601_utc_timestamps() {
    let now_ms = 1_700_000_000_000;
    let ping = build_ping_cot("ALPHA", 1, now_ms);
    let pong = build_pong_cot("ALPHA", "BETA-ping-1", 1, now_ms);
    for xml in [ping, pong] {
        assert!(xml.contains("time=\"2023-11-14T22:13:20Z\""), "{xml}");
        assert!(xml.contains("start=\"2023-11-14T22:13:20Z\""), "{xml}");
        assert!(xml.contains("stale=\"2023-11-14T22:13:30Z\""), "{xml}");
    }
}
