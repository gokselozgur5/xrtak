const ROOT_INDEX: &str = include_str!("../../../index.html");

#[test]
fn telemetry_stream_syncs_terrain_from_world_snapshot() {
    let stream_block = ROOT_INDEX
        .split("const es = new EventSource(apiUrl('/api/stream'));")
        .nth(1)
        .expect("EventSource /api/stream block must exist in index.html")
        .split("es.onerror")
        .next()
        .expect("onmessage block extract");

    assert!(
        stream_block.contains("terrain_name") || stream_block.contains("syncServerTerrain"),
        "EventSource onmessage handler must check terrain_name or call syncServerTerrain to synchronize terrain across clients"
    );
}

#[test]
fn client_fetches_terrain_from_get_api_terrain() {
    assert!(
        ROOT_INDEX.contains("fetch(apiUrl('/api/terrain'))")
            || ROOT_INDEX.contains("fetch(apiUrl('/api/terrain'), {")
            || ROOT_INDEX.contains("syncServerTerrain"),
        "index.html must issue a GET request or call syncServerTerrain to fetch the active synchronized DEM from /api/terrain"
    );
    // Explicitly verify GET /api/terrain is called without method: 'POST' in terrain sync
    let has_get_terrain = ROOT_INDEX.contains("fetch(apiUrl('/api/terrain'))")
        || ROOT_INDEX.contains("fetch(apiUrl('/api/terrain'), { method: 'GET'");
    assert!(
        has_get_terrain,
        "index.html must invoke GET /api/terrain to retrieve synchronized terrain mesh data"
    );
}
