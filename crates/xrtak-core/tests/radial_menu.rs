use xrtak_core::radial::*;

pub fn helper_item(id: &str, act: &str, on: bool) -> RadialMenuItem {
    RadialMenuItem {
        id: id.into(),
        label: id.into(),
        icon: "icon".into(),
        action: RadialAction::SelectTool(act.into()),
        is_enabled: on,
    }
}

pub fn fixture_menu() -> RadialMenuStateMachine {
    let items = vec![
        helper_item("draw", "draw", true),
        helper_item("measure", "measure", true),
        helper_item("off", "off", false),
    ];
    let cfg = RadialMenuConfig {
        inner_deadband_radius: 0.1,
        outer_radius: 0.5,
        dwell_confirm_sec: 0.3,
    };
    RadialMenuStateMachine::new(items, cfg)
}
