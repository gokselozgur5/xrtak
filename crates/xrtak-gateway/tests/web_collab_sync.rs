const ROOT_INDEX: &str = include_str!("../../../index.html");

#[test]
fn sync_collab_state_does_not_reference_undefined_table_scale_factor() {
    let sync_fn = ROOT_INDEX
        .split("async function syncCollabState")
        .nth(1)
        .expect("syncCollabState function must exist in index.html")
        .split("function ")
        .next()
        .expect("function body extract");

    assert!(
        !sync_fn.contains("tableScaleFactor"),
        "syncCollabState must not reference undefined tableScaleFactor identifier"
    );
    assert!(
        sync_fn.contains("tableGroup.scale.x"),
        "syncCollabState must read current scale from tableGroup.scale.x"
    );
}
