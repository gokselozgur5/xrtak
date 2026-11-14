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
