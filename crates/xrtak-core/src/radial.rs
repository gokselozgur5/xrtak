use serde::{Deserialize, Serialize};

/// Action triggered by selecting an item in the tactical radial menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadialAction {
    SelectTool(String),
    ToggleLayer(String),
    QuickCommand(String),
    Custom(String),
}
