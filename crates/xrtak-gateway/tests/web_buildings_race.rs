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
fn buildings_load_guards_against_overlapping_requests_and_hidden_layer() {
    let load_bldgs_fn = ROOT_INDEX
        .split("async function loadRealOsm3dBuildings")
        .nth(1)
        .expect("loadRealOsm3dBuildings function must exist in index.html")
        .split("async function searchAndFlyToCity")
        .next()
        .expect("function body extract");

    // Must track request sequence number or requestId for buildings
    assert!(
        ROOT_INDEX.contains("osmBuildingsSeq") || ROOT_INDEX.contains("bldgsLoadSeq"),
        "index.html must maintain a sequence counter for loadRealOsm3dBuildings requests"
    );

    // loadRealOsm3dBuildings must increment/capture seq
    assert!(
        load_bldgs_fn.contains("osmBuildingsSeq") || load_bldgs_fn.contains("bldgsLoadSeq"),
        "loadRealOsm3dBuildings must check sequence number"
    );

    // clearOsm3dBuildings must invalidate in-flight building loads so canceled/cleared loads do not append late
    let clear_fn = ROOT_INDEX
        .split("function clearOsm3dBuildings")
        .nth(1)
        .expect("clearOsm3dBuildings function must exist in index.html")
        .split("async function loadRealOsm3dBuildings")
        .next()
        .expect("function body extract");

    assert!(
        clear_fn.contains("osmBuildingsSeq") || clear_fn.contains("bldgsLoadSeq"),
        "clearOsm3dBuildings must advance or reset sequence counter to invalidate in-flight requests"
    );

    // After fetch await, must verify sequence is still current and showOsmBuildings is true
    let post_fetch = load_bldgs_fn
        .split("break;\n          }\n        } catch (_) {}")
        .nth(1)
        .expect("fetch loop termination in loadRealOsm3dBuildings");

    assert!(
        post_fetch.contains("!showOsmBuildings"),
        "loadRealOsm3dBuildings must check !showOsmBuildings after fetch await"
    );

    assert!(
        post_fetch.contains("osmBuildingsSeq") || post_fetch.contains("bldgsLoadSeq"),
        "loadRealOsm3dBuildings must check sequence number after fetch await"
    );
}
