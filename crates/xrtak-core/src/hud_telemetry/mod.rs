//! HUD telemetry types and elements.

pub mod battery;
pub mod battery_impl;
pub mod battery_status;
pub mod connection;
pub mod connection_impl;
pub mod element;
pub mod element_impl;
pub mod format;
pub mod types;

pub use battery::BatteryTelemetry;
pub use connection::ConnectionTelemetry;
pub use element::HudTelemetryElement;
pub use types::{BatteryState, ConnectionQuality};
