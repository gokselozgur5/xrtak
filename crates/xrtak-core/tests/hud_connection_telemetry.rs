use xrtak_core::hud_telemetry::connection::ConnectionTelemetry;
use xrtak_core::hud_telemetry::types::ConnectionQuality;

#[test]
fn test_connection_telemetry_disconnected() {
    let conn = ConnectionTelemetry::new(false, 0.0, 0.0);
    assert!(!conn.server_connected);
    assert_eq!(conn.quality, ConnectionQuality::Disconnected);
}

#[test]
fn test_connection_telemetry_quality_levels() {
    let exc = ConnectionTelemetry::new(true, 35.0, 0.1);
    assert_eq!(exc.quality, ConnectionQuality::Excellent);

    let good = ConnectionTelemetry::new(true, 120.0, 1.0);
    assert_eq!(good.quality, ConnectionQuality::Good);

    let poor = ConnectionTelemetry::new(true, 300.0, 20.0);
    assert_eq!(poor.quality, ConnectionQuality::Poor);
}
