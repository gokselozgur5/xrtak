const ROOT_INDEX: &str = include_str!("../../../index.html");

#[test]
fn telemetry_stream_renders_collab_operators_and_applies_table_pose() {
    let msg_handler = ROOT_INDEX
        .split("es.onmessage = (ev) => {")
        .nth(1)
        .expect("es.onmessage handler must exist in index.html")
        .split("};")
        .next()
        .expect("es.onmessage body");

    assert!(
        msg_handler.contains("renderCollabPeers3d") || msg_handler.contains("snap.operators"),
        "telemetry onmessage handler must process snap.operators to render remote collab peers"
    );
    assert!(
        msg_handler.contains("snap.table_pose"),
        "telemetry onmessage handler must process snap.table_pose to synchronize shared sand-table pose"
    );
}

#[test]
fn periodic_collab_sync_heartbeat_keeps_peers_active() {
    assert!(
        ROOT_INDEX.contains("setInterval(syncCollabState") || ROOT_INDEX.contains("setInterval(() => syncCollabState"),
        "index.html must maintain a periodic collab sync interval so operators do not expire after 15s"
    );
}

#[test]
fn ptt_control_and_documentation_describe_transmit_indicator_not_voice_stream() {
    let ptt_btn = ROOT_INDEX
        .split("id=\"btn-collab-ptt\"")
        .nth(1)
        .expect("btn-collab-ptt in index.html")
        .split(">")
        .nth(1)
        .expect("btn inner text")
        .split("<")
        .next()
        .expect("btn text");

    assert!(
        !ptt_btn.to_lowercase().contains("voice"),
        "PTT button must indicate signaling/transmit status rather than claiming voice comms: {ptt_btn}"
    );

    let panel_title = ROOT_INDEX
        .split("MULTI-OPERATOR XR COLLAB")
        .nth(1)
        .expect("collab panel title")
        .split("</div>")
        .next()
        .expect("panel title inner");

    assert!(
        !panel_title.to_lowercase().contains("voice"),
        "Collab panel title must not advertise voice comms: {panel_title}"
    );
}
