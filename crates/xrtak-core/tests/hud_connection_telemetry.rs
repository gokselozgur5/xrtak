use xrtak_core::hud_telemetry::connection::ConnectionTelemetry;
use xrtak_core::hud_telemetry::types::ConnectionQuality;

#[test]
fn test_connection_telemetry_disconnected() {
    let conn = ConnectionTelemetry::new(false, 0.0, 0.0);
    assert!(!conn.server_connected);
    assert_eq!(conn.quality, ConnectionQuality::Disconnected);
}
