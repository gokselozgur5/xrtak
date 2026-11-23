const ROOT_INDEX: &str = include_str!("../../../index.html");

#[test]
fn dem_tile_failures_are_tracked_and_prevent_false_srtm_label() {
    let sector_fn = ROOT_INDEX
        .split("async function loadRealWorldSector")
        .nth(1)
        .expect("loadRealWorldSector function must exist in index.html")
        .split("async function loadRealOsm3dBuildings")
        .next()
        .expect("loadRealWorldSector body extract");

    assert!(
        !sector_fn.contains("let elevM = 850.0;"),
        "loadRealWorldSector must not use a hardcoded 850m elevation fallback for missing DEM samples"
    );

    assert!(
        sector_fn.contains("demLoadedTiles")
            || sector_fn.contains("demFailedTiles")
            || sector_fn.contains("demSuccessCount")
            || sector_fn.contains("demFailures"),
        "loadRealWorldSector must count or track DEM tile load failures"
    );

    assert!(
        sector_fn.contains("FALLBACK") || sector_fn.contains("NO-DEM") || sector_fn.contains("FLAT") || sector_fn.contains("FAILED"),
        "loadRealWorldSector must reflect fallback/failure in the terrain label when DEM tiles fail to load instead of claiming real SRTM"
    );
}
