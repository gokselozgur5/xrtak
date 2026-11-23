const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn head_enu_encoding_uses_consistent_m_to_table_scale() {
    assert_eq!(
        ROOT_INDEX, STATIC_INDEX,
        "root index.html and static/index.html must be byte-identical"
    );

    let sync_fn = ROOT_INDEX
        .split("async function syncCollabState")
        .nth(1)
        .expect("syncCollabState function must exist in index.html")
        .split("function ")
        .next()
        .expect("function body extract");

    assert!(
        !sync_fn.contains("800.0"),
        "syncCollabState must not use arbitrary 800.0 scale factor for horizontal head_enu coordinates"
    );
    assert!(
        !sync_fn.contains("500.0"),
        "syncCollabState must not use arbitrary 500.0 scale factor for vertical head_enu coordinate"
    );
    assert!(
        sync_fn.contains("camera.position.x / M_TO_TABLE")
            || sync_fn.contains("camera.position.x / (M_TO_TABLE"),
        "syncCollabState must scale horizontal camera x by M_TO_TABLE"
    );
    assert!(
        sync_fn.contains("camera.position.z / M_TO_TABLE")
            || sync_fn.contains("camera.position.z / (M_TO_TABLE"),
        "syncCollabState must scale horizontal camera z by M_TO_TABLE"
    );
    assert!(
        sync_fn.contains("ALT_EXAGGERATION"),
        "syncCollabState must scale vertical camera y consistently with ALT_EXAGGERATION"
    );
}
