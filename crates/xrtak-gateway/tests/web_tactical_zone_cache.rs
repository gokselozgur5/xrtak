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
fn tactical_zone_cache_key_includes_vertex_coordinates() {
    let zones_sig_extract = ROOT_INDEX
        .split("if (showTacticalZones && Array.isArray(zones)) {")
        .nth(1)
        .expect("Tactical zones block must exist in index.html")
        .split("if (zonesSig !== lastTacticalZonesSig)")
        .next()
        .expect("zonesSig assignment must precede cache comparison");

    // The signature must incorporate vertex coordinates (not just length) so that moving
    // geofence vertices invalidates the stale cache outline.
    assert!(
        zones_sig_extract.contains("v[0]") || zones_sig_extract.contains("Math.round(v[0])") || zones_sig_extract.contains("v[1]"),
        "Tactical zone cache signature must include vertex coordinate positions (e.g. v[0], v[1]) to detect moved geofences: {zones_sig_extract}"
    );
}
