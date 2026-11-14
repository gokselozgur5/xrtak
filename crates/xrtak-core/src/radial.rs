use serde::{Deserialize, Serialize};

/// Action triggered by selecting an item in the tactical radial menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadialAction {
    SelectTool(String),
    ToggleLayer(String),
    QuickCommand(String),
    Custom(String),
}

/// Item displayed in a slice of the radial menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadialMenuItem {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub action: RadialAction,
    pub is_enabled: bool,
}

/// State of the radial menu interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadialMenuState {
    Closed,
    Opening,
    Active,
    Confirming,
    Dismissed,
}

/// Configuration parameters for radial menu geometry and thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialMenuConfig {
    pub inner_deadband_radius: f64,
    pub outer_radius: f64,
    pub dwell_confirm_sec: f64,
}

impl Default for RadialMenuConfig {
    fn default() -> Self {
        Self {
            inner_deadband_radius: 0.05,
            outer_radius: 0.35,
            dwell_confirm_sec: 0.40,
        }
    }
}
