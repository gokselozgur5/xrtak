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
fn roster_rendering_does_not_use_inner_html_for_entity_fields() {
    let roster_fn = ROOT_INDEX
        .split("function updateHudAndRoster")
        .nth(1)
        .expect("updateHudAndRoster function must exist in index.html")
        .split("function ")
        .next()
        .expect("function body extract");

    assert!(
        !roster_fn.contains("item.innerHTML"),
        "unitListEl items must not be populated via innerHTML"
    );
    assert!(
        !roster_fn.contains("${ent.callsign}"),
        "callsign must not be interpolated into HTML strings"
    );
    assert!(
        !roster_fn.contains("${ent.cot_type}"),
        "cot_type must not be interpolated into HTML strings"
    );
}
