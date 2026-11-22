use xrtak_core::geochat::{
    GeoChatPacket, GeoChatThreader, DEFAULT_MAX_THREADS, DEFAULT_MAX_THREAD_MESSAGES,
    DEFAULT_MAX_TOTAL_MESSAGES,
};

fn pkt(uid: &str, room: &str, mid: &str) -> GeoChatPacket {
    GeoChatPacket::new(uid, uid, room, mid, "msg", "2026-11-18T10:00:00Z")
}

#[test]
fn redelivered_message_is_not_threaded_twice() {
    let mut threader = GeoChatThreader::new();
    let msg = pkt("U-1", "HQ", "M-1");
    assert_eq!(threader.ingest(msg.clone()), "HQ");
    assert_eq!(threader.ingest(msg), "HQ");
    assert_eq!(threader.thread("HQ").expect("thread").message_count(), 1);
}

#[test]
fn per_thread_retention_evicts_oldest_and_rebuilds_participants() {
    let mut threader = GeoChatThreader::new();
    let oldest = pkt("U-OLD", "HQ", "M-0").add_recipient("U-RECIP-OLD");
    threader.ingest(oldest.clone());
    for i in 1..=300 {
        threader.ingest(pkt("U-ACT", "HQ", &format!("M-{i}")));
    }
    let hq = threader.thread("HQ").expect("thread");
    assert!(hq.message_count() <= 256);
    assert_eq!(hq.participants, vec!["U-ACT".to_string()]);
    assert_eq!(threader.all_participants(), vec!["U-ACT".to_string()]);
    threader.ingest(oldest);
    let hq = threader.thread("HQ").unwrap();
    assert_eq!(hq.last_message().unwrap().message_id, "M-0");
}

#[test]
fn global_retention_bounds_threads_and_message_index() {
    let mut threader = GeoChatThreader::new();
    threader.ingest(pkt("U-Z", "ROOM-0", "G-0"));
    for i in 1..=300 {
        threader.ingest(pkt("U-N", &format!("ROOM-{i}"), &format!("G-{i}")));
    }
    assert!(threader.thread_count() <= 256);
    assert!(threader.thread("ROOM-0").is_none());
    assert_eq!(threader.all_participants(), vec!["U-N".to_string()]);
    let mut reply = pkt("U-N", "ALT", "G-R");
    reply.parent_message_id = Some("G-0".to_string());
    assert_eq!(threader.ingest(reply), "G-0");
}

#[test]
fn custom_retention_limits_trim_emptied_conversations_and_clamp_zero() {
    let def = GeoChatThreader::default();
    assert_eq!(def.max_threads(), DEFAULT_MAX_THREADS);
    assert_eq!(def.max_thread_messages(), DEFAULT_MAX_THREAD_MESSAGES);
    assert_eq!(def.max_total_messages(), DEFAULT_MAX_TOTAL_MESSAGES);

    let clamped = GeoChatThreader::with_retention_limits(0, 0, 0);
    assert_eq!(
        (
            clamped.max_threads(),
            clamped.max_thread_messages(),
            clamped.max_total_messages()
        ),
        (1, 1, 1)
    );

    let mut t = GeoChatThreader::with_retention_limits(4, 3, 3);
    t.ingest(pkt("U-A", "HQ", "M-1").with_conversation("C-1", None));
    t.ingest(pkt("U-B", "HQ", "M-2").with_conversation("C-1", None));
    t.ingest(pkt("U-C", "HQ", "M-3").with_conversation("C-2", None));
    t.ingest(pkt("U-D", "HQ", "M-4").with_conversation("C-3", None));
    t.ingest(pkt("U-E", "HQ", "M-5").with_conversation("C-3", None));

    assert!(t.thread("C-1").is_none());
    assert_eq!(t.thread_count(), 2);
    assert_eq!(t.retained_message_count(), 3);
    assert_eq!(t.all_participants(), vec!["U-C", "U-D", "U-E"]);

    let dup = GeoChatPacket::new("U-X", "X", "HQ", "M-3", "altered", "2026-11-18T10:05:00Z");
    assert_eq!(t.ingest(dup), "C-2");
    assert_eq!(t.retained_message_count(), 3);
    assert_eq!(t.thread("C-2").unwrap().messages[0].message_text, "msg");
}
