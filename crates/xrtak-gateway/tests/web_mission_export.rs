const ROOT_INDEX: &str = include_str!("../../../index.html");

#[test]
fn export_mission_route_does_not_send_hardcoded_demo_waypoints() {
    let export_fn = ROOT_INDEX
        .split("async function exportMissionRoute")
        .nth(1)
        .expect("exportMissionRoute function must exist in index.html")
        .split("document.getElementById('btn-export-qgc')")
        .next()
        .expect("function body extract");

    assert!(
        !export_fn.contains("-360.0") && !export_fn.contains("380.0"),
        "exportMissionRoute must not substitute hardcoded demo coordinates when no mission is loaded"
    );
    assert!(
        export_fn.contains("if (wpsEnAgl.length < 2)") || export_fn.contains("return;"),
        "exportMissionRoute must refuse export when there are insufficient route waypoints"
    );
}
