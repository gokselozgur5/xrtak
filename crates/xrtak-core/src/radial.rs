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

/// Tactical radial menu state machine managing gesture inputs.
#[derive(Debug, Clone)]
pub struct RadialMenuStateMachine {
    pub config: RadialMenuConfig,
    pub items: Vec<RadialMenuItem>,
    pub state: RadialMenuState,
    pub selected_index: Option<usize>,
    pub dwell_timer_sec: f64,
}

impl RadialMenuStateMachine {
    pub fn new(items: Vec<RadialMenuItem>, config: RadialMenuConfig) -> Self {
        Self {
            config,
            items,
            state: RadialMenuState::Closed,
            selected_index: None,
            dwell_timer_sec: 0.0,
        }
    }
}

impl RadialMenuStateMachine {
    /// Opens the radial menu if currently closed.
    pub fn open(&mut self) {
        if self.state == RadialMenuState::Closed || self.state == RadialMenuState::Dismissed {
            self.state = RadialMenuState::Active;
            self.selected_index = None;
            self.dwell_timer_sec = 0.0;
        }
    }
}

impl RadialMenuStateMachine {
    /// Dismisses the radial menu without executing any action.
    pub fn dismiss(&mut self) {
        self.state = RadialMenuState::Dismissed;
        self.selected_index = None;
        self.dwell_timer_sec = 0.0;
    }
}
