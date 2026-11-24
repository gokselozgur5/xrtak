const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn github_pages_demo_sim_configuration_and_sri_integrity() {
    assert_eq!(
        ROOT_INDEX, STATIC_INDEX,
        "root index.html and crates/xrtak-gateway/static/index.html must be identical"
    );

    // Hardcoded private LAN IP must be completely removed
    assert!(
        !ROOT_INDEX.contains("192.168.1.6"),
        "index.html must not contain hardcoded private LAN IP 192.168.1.6"
    );

    // Must detect static demo host (github.io or file:) without gateway param
    assert!(
        ROOT_INDEX.contains("window.location.hostname.endsWith('github.io')")
            || ROOT_INDEX.contains("window.location.hostname.endsWith(\"github.io\")"),
        "index.html must detect github.io host"
    );
    assert!(
        ROOT_INDEX.contains("window.location.protocol === 'file:'")
            || ROOT_INDEX.contains("window.location.protocol === \"file:\""),
        "index.html must detect file: protocol"
    );

    // DEMO SIM badge must be shown when standalone client simulation runs
    assert!(
        ROOT_INDEX.contains("DEMO SIM"),
        "index.html must display DEMO SIM badge for standalone client sim"
    );

    // Three.js module must be used with verified SHA-384 SRI integrity and crossorigin attributes
    assert!(
        !ROOT_INDEX.contains("three.min.js"),
        "index.html must not reference deprecated or non-existent three.min.js"
    );
    assert!(
        ROOT_INDEX.contains("three.module.js"),
        "index.html must reference three.module.js"
    );
    assert!(
        ROOT_INDEX.contains(
            "integrity=\"sha384-wlK04bYFr3nyu6ipQq5N4n7KKGByMS1Yu1T8IrNtEnHiJ5WcGDt9dZBZFpRZjXEE\""
        ),
        "index.html must specify verified SHA-384 integrity hash for three.module.js"
    );
    assert!(
        ROOT_INDEX.contains("crossorigin=\"anonymous\"")
            || ROOT_INDEX.contains("crossorigin='anonymous'"),
        "index.html must specify crossorigin anonymous attribute"
    );

    // Importmap must be declared strictly before modulepreload and module scripts (WHATWG / Safari requirement)
    let importmap_idx = ROOT_INDEX
        .find("<script type=\"importmap\">")
        .expect("index.html must declare importmap");
    let modulepreload_idx = ROOT_INDEX
        .find("<link rel=\"modulepreload\"")
        .expect("index.html must declare modulepreload link");
    let module_idx = ROOT_INDEX
        .find("<script type=\"module\">")
        .expect("index.html must declare module script");

    assert!(
        importmap_idx < modulepreload_idx,
        "importmap must precede modulepreload link"
    );
    assert!(
        importmap_idx < module_idx,
        "importmap must precede module script"
    );
    // High-resolution satellite imagery configuration decoupled from DEM zoom
    assert!(
        ROOT_INDEX.contains("getMaxAnisotropy"),
        "index.html must configure max anisotropic filtering on terrain satellite texture"
    );
    assert!(
        ROOT_INDEX.contains("2048"),
        "index.html must crop satellite texture to 2048x2048 canvas for razor-sharp HD rendering"
    );
    assert!(
        ROOT_INDEX.contains("zSat") && ROOT_INDEX.contains("zDem"),
        "index.html must decouple satellite imagery zoom (zSat) from elevation zoom (zDem)"
    );

    // Spatial grab ring and SPI reticle orientation regression checks
    assert!(
        !ROOT_INDEX.contains("spatialGrabRing.rotation.z"),
        "spatialGrabRing must not rotate around Z axis"
    );
    assert!(
        ROOT_INDEX.contains("spatialGrabRing.rotation.y"),
        "spatialGrabRing must rotate around Y axis"
    );
    assert!(
        ROOT_INDEX.contains("spatialGrabRing.visible = false"),
        "spatialGrabRing must default to hidden on desktop"
    );
    assert!(
        ROOT_INDEX.contains("renderer.xr.isPresenting"),
        "spatialGrabRing visibility must be gated on XR presentation"
    );
    assert!(
        !ROOT_INDEX.contains("node.iconMesh.rotation.z"),
        "SPI reticle iconMesh must not rotate around Z axis"
    );
    assert!(
        ROOT_INDEX.contains("node.iconMesh.rotation.y += 0.04"),
        "SPI reticle iconMesh must rotate around Y axis"
    );
}
