use std::fs;

#[test]
fn xr_sand_table_controller_node_paths_and_state_polling() {
    let script_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../godot-xr/scripts/xr_sand_table.gd"
    );
    let content =
        fs::read_to_string(script_path).expect("godot-xr/scripts/xr_sand_table.gd must exist");

    assert!(
        content.contains("get_node_or_null(\"XROrigin3D/LeftHand\")"),
        "xr_sand_table.gd must resolve LeftHand to match main_xr.tscn"
    );
    assert!(
        content.contains("get_node_or_null(\"XROrigin3D/RightHand\")"),
        "xr_sand_table.gd must resolve RightHand to match main_xr.tscn"
    );
    assert!(
        !content.contains("LeftHandController"),
        "xr_sand_table.gd must not refer to obsolete LeftHandController"
    );
    assert!(
        content.contains("http_state_req.request("),
        "xr_sand_table.gd must poll /api/state via http_state_req.request"
    );
}

#[test]
fn xr_sand_table_walk_in_toggle_scale_matches_m_to_table_scale() {
    let script_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../godot-xr/scripts/xr_sand_table.gd"
    );
    let content =
        fs::read_to_string(script_path).expect("godot-xr/scripts/xr_sand_table.gd must exist");

    assert!(
        content.contains("1.0 / m_to_table_scale"),
        "walk-in scale must derive from 1.0 / m_to_table_scale rather than hardcoded 10.0"
    );
    assert!(
        !content.contains("10.0 if is_walk_in_scale else 1.0"),
        "xr_sand_table.gd must not use incorrect hardcoded 10.0 walk-in scale"
    );
}
