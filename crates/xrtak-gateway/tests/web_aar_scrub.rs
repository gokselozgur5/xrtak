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
fn aar_replay_guard_queues_pending_scrub_while_fetch_in_flight() {
    let fn_body = ROOT_INDEX
        .split("async function loadAarFrameAt")
        .nth(1)
        .expect("loadAarFrameAt function must exist in index.html")
        .split("aarSlider.addEventListener")
        .next()
        .expect("function body extract");

    // Must have pending scrub tracking or queueing
    assert!(
        ROOT_INDEX.contains("aarPendingSeek")
            || ROOT_INDEX.contains("pendingAarSeek")
            || ROOT_INDEX.contains("aarPendingTime"),
        "index.html must track pending AAR scrub/seek target when fetch is in flight"
    );

    // Guard in loadAarFrameAt must record requested time if fetch is in flight
    assert!(
        fn_body.contains("aarPendingSeek = t")
            || fn_body.contains("pendingAarSeek = t")
            || fn_body.contains("aarPendingTime = t"),
        "loadAarFrameAt must record pending seek target when returning early"
    );

    // Finally block or post-fetch must dispatch pending seek
    assert!(
        fn_body.contains("loadAarFrameAt(")
            && (fn_body.contains("aarPendingSeek")
                || fn_body.contains("pendingAarSeek")
                || fn_body.contains("aarPendingTime")),
        "loadAarFrameAt must re-trigger pending seek once in-flight fetch completes"
    );
}
