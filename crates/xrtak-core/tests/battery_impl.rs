use xrtak_core::hud_telemetry::battery::BatteryTelemetry;
use xrtak_core::hud_telemetry::connection::ConnectionTelemetry;
use xrtak_core::hud_telemetry::types::{BatteryState, ConnectionQuality};

#[test]
fn test_battery_critical_threshold_alignment() {
    let bat = BatteryTelemetry::new(12.0, 14.8, false);
    assert_eq!(bat.state == BatteryState::Critical, bat.is_critical());
}

#[test]
fn test_battery_nan_input_handling() {
    let bat = BatteryTelemetry::new(f32::NAN, 12.0, false);
    assert_eq!(bat.percentage, 0.0);
    assert_eq!(bat.state, BatteryState::Critical);
    assert!(bat.is_critical());
}

#[test]
fn test_connection_nan_input_handling() {
    let conn_lat = ConnectionTelemetry::new(true, f32::NAN, 0.0);
    assert_eq!(conn_lat.quality, ConnectionQuality::Disconnected);

    let conn_loss = ConnectionTelemetry::new(true, 50.0, f32::NAN);
    assert_eq!(conn_loss.quality, ConnectionQuality::Disconnected);
}
