const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn built_in_simulation_aligns_with_gateway_schema() {
    assert_eq!(
        ROOT_INDEX, STATIC_INDEX,
        "root index.html and crates/xrtak-gateway/static/index.html must be identical"
    );

    let sim_fn = ROOT_INDEX
        .split("function startStandaloneClientSim(")
        .nth(1)
        .expect("startStandaloneClientSim must exist in index.html")
        .split("function connectTelemetryStream()")
        .next()
        .expect("startStandaloneClientSim body");

    // Must not produce lat_deg or lon_deg in entity objects
    assert!(
        !sim_fn.contains("lat_deg:"),
        "standalone simulation entities must use 'lat:' instead of 'lat_deg:'"
    );
    assert!(
        !sim_fn.contains("lon_deg:"),
        "standalone simulation entities must use 'lon:' instead of 'lon_deg:'"
    );

    // Must not produce capitalised affiliation strings like 'Friendly' or 'Hostile'
    assert!(
        !sim_fn.contains("affiliation: 'Friendly'"),
        "standalone simulation entities must use lower-case affiliation 'friendly'"
    );
    assert!(
        !sim_fn.contains("affiliation: 'Hostile'"),
        "standalone simulation entities must use lower-case affiliation 'hostile'"
    );

    // Tactical zones must use vertices_en and lower-case affiliation
    assert!(
        !sim_fn.contains("vertices_enu:"),
        "tactical zones must use 'vertices_en:' matching gateway snapshot schema"
    );
    assert!(
        sim_fn.contains("vertices_en:"),
        "tactical zones must specify 'vertices_en:'"
    );
    assert!(
        sim_fn.contains("affiliation: 'friendly'") || sim_fn.contains("affiliation: 'hostile'"),
        "tactical zones must specify lower-case affiliation"
    );
}
