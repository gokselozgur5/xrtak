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
fn apogee_marker_placed_at_maximum_altitude_sample_not_midpoint() {
    let fires_extract = ROOT_INDEX
        .split("const arcLine = new THREE.Line")
        .nth(1)
        .expect("Trajectory line creation must exist in index.html")
        .split("tableGroup.add(ballisticsMeshGroup);")
        .next()
        .expect("tableGroup.add(ballisticsMeshGroup) must follow apogee marker creation");

    assert!(
        !fires_extract.contains("Math.floor(pts.length / 2)"),
        "Apogee marker must not use flight midpoint Math.floor(pts.length / 2): {fires_extract}"
    );

    assert!(
        fires_extract.contains("up_m")
            || fires_extract.contains("maxUp")
            || fires_extract.contains("max_ordinate"),
        "Apogee marker must find apex by maximum up_m altitude: {fires_extract}"
    );
}
