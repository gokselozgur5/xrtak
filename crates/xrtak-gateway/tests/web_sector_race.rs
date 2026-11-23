const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn sector_load_uses_sequence_guard_to_prevent_stale_overwrites() {
    assert_eq!(
        ROOT_INDEX, STATIC_INDEX,
        "root index.html and static/index.html must be byte-identical"
    );

    let load_fn = ROOT_INDEX
        .split("async function loadRealWorldSector")
        .nth(1)
        .expect("loadRealWorldSector function must exist in index.html")
        .split("function ")
        .next()
        .expect("function body extract");

    // Must track request sequence number or requestId
    assert!(
        ROOT_INDEX.contains("sectorLoadSeq") || ROOT_INDEX.contains("realWorldSectorSeq"),
        "index.html must maintain a sequence counter for loadRealWorldSector requests"
    );

    // loadRealWorldSector must increment/capture seq and check it after tile fetch await
    assert!(
        load_fn.contains("sectorLoadSeq") || load_fn.contains("realWorldSectorSeq"),
        "loadRealWorldSector must check sequence number after async tile fetches"
    );

    // Module-level origin must NOT be committed before awaiting tileJobs, or must be guarded
    // When stale request completes, it must abort before updating activeDemGrid, origin or tak-config
    let post_await = load_fn
        .split("await Promise.all(tileJobs);")
        .nth(1)
        .expect("await Promise.all(tileJobs) must exist in loadRealWorldSector");

    assert!(
        post_await.contains("return") || post_await.contains("sectorLoadSeq"),
        "loadRealWorldSector must discard stale results after Promise.all(tileJobs)"
    );
}
