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

impl RadialMenuStateMachine {
    /// Evaluates cursor distance from menu center.
    pub fn is_within_bounds(&self, x: f64, y: f64) -> bool {
        let dist = (x * x + y * y).sqrt();
        dist >= self.config.inner_deadband_radius && dist <= self.config.outer_radius
    }
}

impl RadialMenuStateMachine {
    /// Maps 2D coordinate to sector index if items present.
    pub fn sector_index_at(&self, x: f64, y: f64) -> Option<usize> {
        if self.items.is_empty() || !self.is_within_bounds(x, y) {
            return None;
        }
        let angle = y.atan2(x).to_degrees().rem_euclid(360.0);
        let sector_size = 360.0 / (self.items.len() as f64);
        Some(((angle / sector_size).floor() as usize) % self.items.len())
    }
}

impl RadialMenuStateMachine {
    /// Updates cursor position and accumulates dwell time.
    pub fn update_cursor(&mut self, x: f64, y: f64, dt_sec: f64) {
        if self.state != RadialMenuState::Active && self.state != RadialMenuState::Confirming {
            return;
        }
        let target_idx = self.sector_index_at(x, y);
        self.handle_sector_hover(target_idx, dt_sec);
    }
}

impl RadialMenuStateMachine {
    fn handle_sector_hover(&mut self, target_idx: Option<usize>, dt_sec: f64) {
        match (self.selected_index, target_idx) {
            (Some(cur), Some(tgt)) if cur == tgt => {
                self.dwell_timer_sec += dt_sec;
                if self.dwell_timer_sec >= self.config.dwell_confirm_sec {
                    self.state = RadialMenuState::Confirming;
                }
            }
            _ => {
                self.selected_index = target_idx;
                self.dwell_timer_sec = 0.0;
                self.state = RadialMenuState::Active;
            }
        }
    }
}

impl RadialMenuStateMachine {
    /// Commits selection if hovering an enabled item, closing menu.
    pub fn select(&mut self) -> Option<RadialAction> {
        if self.state != RadialMenuState::Active && self.state != RadialMenuState::Confirming {
            return None;
        }
        let action = self.selected_index.and_then(|idx| {
            self.items.get(idx).and_then(|item| {
                if item.is_enabled {
                    Some(item.action.clone())
                } else {
                    None
                }
            })
        });
        if action.is_some() {
            self.state = RadialMenuState::Closed;
            self.selected_index = None;
            self.dwell_timer_sec = 0.0;
        }
        action
    }
}
