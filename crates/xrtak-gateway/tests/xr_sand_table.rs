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
