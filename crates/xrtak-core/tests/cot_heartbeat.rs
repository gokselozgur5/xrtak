use xrtak_core::{build_ping_cot, build_pong_cot};

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
