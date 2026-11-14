use xrtak_core::hud_telemetry::battery::BatteryTelemetry;
use xrtak_core::hud_telemetry::types::BatteryState;

#[test]
fn test_battery_telemetry_discharging() {
    let bat = BatteryTelemetry::new(75.0, 3.85, false);
    assert_eq!(bat.state, BatteryState::Discharging);
    assert!(!bat.is_low());
    assert!(!bat.is_critical());
}
