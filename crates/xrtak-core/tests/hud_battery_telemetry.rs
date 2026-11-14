use xrtak_core::hud_telemetry::battery::BatteryTelemetry;
use xrtak_core::hud_telemetry::types::BatteryState;

#[test]
fn test_battery_telemetry_discharging() {
    let bat = BatteryTelemetry::new(75.0, 3.85, false);
    assert_eq!(bat.state, BatteryState::Discharging);
    assert!(!bat.is_low());
    assert!(!bat.is_critical());
}

#[test]
fn test_battery_telemetry_charging_and_full() {
    let bat_chg = BatteryTelemetry::new(50.0, 4.10, true);
    assert_eq!(bat_chg.state, BatteryState::Charging);

    let bat_full = BatteryTelemetry::new(99.5, 4.20, true);
    assert_eq!(bat_full.state, BatteryState::Full);
}

#[test]
fn test_battery_telemetry_low_and_critical_thresholds() {
    let bat_low = BatteryTelemetry::new(18.0, 3.45, false);
    assert!(bat_low.is_low());
    assert!(!bat_low.is_critical());

    let bat_crit = BatteryTelemetry::new(8.0, 3.20, false);
    assert_eq!(bat_crit.state, BatteryState::Critical);
    assert!(bat_crit.is_low());
    assert!(bat_crit.is_critical());
}
