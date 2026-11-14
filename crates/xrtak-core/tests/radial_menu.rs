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

#[test]
fn test_open_and_dismiss_flow() {
    let mut menu = fixture_menu();
    assert_eq!(menu.state, RadialMenuState::Closed);
    menu.open();
    assert_eq!(menu.state, RadialMenuState::Active);
    assert_eq!(menu.selected_index, None);
    menu.dismiss();
    assert_eq!(menu.state, RadialMenuState::Dismissed);
    menu.open();
    assert_eq!(menu.state, RadialMenuState::Active);
}

#[test]
fn test_deadband_and_bounds_detection() {
    let mut menu = fixture_menu();
    menu.open();
    menu.update_cursor(0.05, 0.0, 0.1);
    assert_eq!(menu.selected_index, None);
    menu.update_cursor(0.6, 0.0, 0.1);
    assert_eq!(menu.selected_index, None);
    menu.update_cursor(0.2, 0.0, 0.1);
    assert_eq!(menu.selected_index, Some(0));
}
