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
fn aar_slider_step_allows_tenth_second_increments_for_playback() {
    let slider_tag = ROOT_INDEX
        .lines()
        .find(|line| line.contains("id=\"aar-slider\""))
        .expect("input range element with id aar-slider must exist");

    assert!(
        slider_tag.contains("step=\"0.1\""),
        "aar-slider must have step=\"0.1\" so 0.1s increments at 1x speed do not snap to 0.2: {slider_tag}"
    );
}
