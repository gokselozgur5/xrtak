use std::fs;

#[test]
fn xr_sand_table_no_hardcoded_private_lan_ips_and_configurable_gateway() {
    let script_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../godot-xr/scripts/xr_sand_table.gd"
    );
    let content =
        fs::read_to_string(script_path).expect("godot-xr/scripts/xr_sand_table.gd must exist");

    // Ensure no hardcoded private LAN IPs
    assert!(
        !content.contains("192.168."),
        "xr_sand_table.gd must not contain hardcoded 192.168.x.x LAN IPs"
    );
    assert!(
        !content.contains("10.0.") && !content.contains("172.16."),
        "xr_sand_table.gd must not contain hardcoded private network IPs"
    );

    // Ensure configurable via XRTAK_GATEWAY_URL and CLI argument
    assert!(
        content.contains("OS.get_environment(\"XRTAK_GATEWAY_URL\")"),
        "xr_sand_table.gd must check XRTAK_GATEWAY_URL environment variable"
    );
    assert!(
        content.contains("--gateway="),
        "xr_sand_table.gd must check --gateway= command line argument"
    );
    assert!(
        content.contains("127.0.0.1:8090"),
        "xr_sand_table.gd must provide default fallback to 127.0.0.1:8090"
    );
}

#[test]
fn default_action_map_defines_openxr_actions_and_touch_controller_profile() {
    let map_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../godot-xr/default_action_map.tres"
    );
    let content =
        fs::read_to_string(map_path).expect("godot-xr/default_action_map.tres must exist");

    assert!(
        content.contains("[gd_resource type=\"OpenXRActionMap\" load_steps=27"),
        "must declare OpenXRActionMap with accurate load_steps=27"
    );
    assert!(
        content.contains("resource_name = \"grip_click\""),
        "must define grip_click action"
    );
    assert!(
        content.contains("resource_name = \"trigger_click\""),
        "must define trigger_click action"
    );
    assert!(
        content.contains("resource_name = \"ax_button\""),
        "must define ax_button action"
    );
    assert!(
        content.contains("resource_name = \"by_button\""),
        "must define by_button action"
    );
    assert!(
        content.contains(
            "interaction_profile_path = \"/interaction_profiles/oculus/touch_controller\""
        ),
        "must define Oculus Touch controller interaction profile"
    );
}
