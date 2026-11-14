use xrtak_core::hud_telemetry::battery::BatteryTelemetry;
use xrtak_core::hud_telemetry::connection::ConnectionTelemetry;
use xrtak_core::hud_telemetry::element::HudTelemetryElement;

#[test]
fn test_hud_telemetry_element_format() {
    let bat = BatteryTelemetry::new(82.0, 3.90, false);
    let conn = ConnectionTelemetry::new(true, 42.0, 0.0);
    let hud = HudTelemetryElement::new("ALPHA-1", bat, conn);

    let status = hud.format_status_line();
    assert!(status.contains("ALPHA-1"));
    assert!(status.contains("82%"));
    assert!(status.contains("EXCELLENT"));
}
