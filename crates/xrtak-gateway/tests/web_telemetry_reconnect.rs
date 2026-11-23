const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn telemetry_stream_reconnects_and_does_not_switch_to_simulated_tracks() {
    assert_eq!(ROOT_INDEX, STATIC_INDEX);

    let stream_fn = ROOT_INDEX
        .split("function connectTelemetryStream()")
        .nth(1)
        .expect("connectTelemetryStream function must exist in index.html")
        .split("connectTelemetryStream();")
        .next()
        .expect("connectTelemetryStream body extract");

    assert!(
        !stream_fn.contains("startStandaloneClientSim"),
        "telemetry stream error handler must not fallback to startStandaloneClientSim"
    );

    assert!(
        !stream_fn.contains("es.close()"),
        "EventSource must not be explicitly closed on error so browser can reconnect"
    );

    assert!(
        stream_fn.contains("LINK-LOST") || stream_fn.contains("LINK LOST"),
        "telemetry stream must indicate link-lost state on stream error"
    );

    assert!(
        stream_fn.contains("es.onopen ="),
        "EventSource must register onopen to restore live indicator when stream reconnects"
    );
}
