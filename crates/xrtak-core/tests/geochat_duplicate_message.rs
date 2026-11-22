use xrtak_core::geochat::{GeoChatPacket, GeoChatThreader};

#[test]
fn redelivered_message_is_not_threaded_twice() {
    let pkt = GeoChatPacket::new("U-1", "LEAD", "HQ", "M-1", "hold", "2026-11-18T10:00:00Z");
    let mut threader = GeoChatThreader::new();
    threader.ingest(pkt.clone());
    threader.ingest(pkt);
    let thread = threader.thread("HQ").expect("thread exists");
    assert_eq!(thread.message_count(), 1);
}
