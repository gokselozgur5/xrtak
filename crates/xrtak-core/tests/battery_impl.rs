use xrtak_core::hud_telemetry::battery::BatteryTelemetry;
use xrtak_core::hud_telemetry::types::BatteryState;

#[test]
fn test_battery_critical_threshold_alignment() {
    let bat = BatteryTelemetry::new(12.0, 14.8, false);
    assert_eq!(bat.state == BatteryState::Critical, bat.is_critical());
}
