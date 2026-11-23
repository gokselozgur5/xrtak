const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn index_html_copies_are_byte_identical() {
    assert_eq!(
        ROOT_INDEX, STATIC_INDEX,
        "root index.html and static/index.html must be byte-identical"
    );
}

#[test]
fn buildings_cleared_on_sector_load_and_reloaded_on_origin_change() {
    let load_sector_fn = ROOT_INDEX
        .split("async function loadRealWorldSector")
        .nth(1)
        .expect("loadRealWorldSector function must exist in index.html")
        .split("function ")
        .next()
        .expect("function body extract");

    assert!(
        load_sector_fn.contains("clearOsm3dBuildings()"),
        "loadRealWorldSector must call clearOsm3dBuildings() unconditionally when sector loads"
    );

    let toggle_fn = ROOT_INDEX
        .split("function toggleOsm3dBuildings")
        .nth(1)
        .expect("toggleOsm3dBuildings function must exist in index.html")
        .split("document.getElementById")
        .next()
        .expect("function body extract");

    assert!(
        toggle_fn.contains("osmBuildingsOrigin") || toggle_fn.contains("osmBuildingsGroup"),
        "toggleOsm3dBuildings must check if buildings belong to current sector origin"
    );

    assert!(
        !toggle_fn.contains("showOsmBuildings && osmBuildingsGroup.children.length === 0\n")
            && !toggle_fn.contains("showOsmBuildings && osmBuildingsGroup.children.length === 0 {"),
        "toggleOsm3dBuildings must not solely rely on children.length === 0 without origin verification: {toggle_fn}"
    );
}
