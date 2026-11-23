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
}
