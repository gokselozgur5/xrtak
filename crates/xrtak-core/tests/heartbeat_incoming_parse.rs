use xrtak_core::{CotHeartbeatManager, HeartbeatAction};

#[test]
fn delete_event_is_not_treated_as_ping() {
    let mut mgr = CotHeartbeatManager::default();
    let delete = r#"<event version="2.0" uid="shipping-yard-7" type="t-x-d-d" how="m-g"><point lat="0" lon="0" hae="0" ce="9999999" le="9999999"/><detail><link uid="shipping-yard-7" relation="none" type="none"/><__forcedelete/></detail></event>"#;
    assert_eq!(
        mgr.process_incoming_cot(delete, 1_000),
        HeartbeatAction::Ignored
    );
}

#[test]
fn single_quoted_ping_is_answered() {
    let mut mgr = CotHeartbeatManager::default();
    let ping = "<event version='2.0' uid='NODE-ping' type='t-x-c-t' how='m-g'/>";
    let action = mgr.process_incoming_cot(ping, 1_000);
    assert!(
        matches!(action, HeartbeatAction::RespondWithPong(_)),
        "{action:?}"
    );
}
