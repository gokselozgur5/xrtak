const ROOT_INDEX: &str = include_str!("../../../index.html");
const STATIC_INDEX: &str = include_str!("../static/index.html");

#[test]
fn plan_offline_mapset_reads_declared_imagery_style_inside_try_block() {
    assert_eq!(ROOT_INDEX, STATIC_INDEX);
    assert!(
        !ROOT_INDEX.contains("activeImageryMode") && !STATIC_INDEX.contains("activeImageryMode"),
        "index.html copies must not reference undeclared activeImageryMode"
    );
    assert!(
        ROOT_INDEX.contains("let currentImageryStyle ="),
        "currentImageryStyle state variable must be declared in index.html"
    );

    let handler = ROOT_INDEX
        .split("document.getElementById('btn-plan-mapset').onclick")
        .nth(1)
        .expect("btn-plan-mapset onclick handler must exist in index.html")
        .split("document.getElementById('btn-eval-rf')")
        .next()
        .expect("btn-plan-mapset handler body extract");

    let (before_try, inside_try) = handler
        .split_once("try {")
        .expect("btn-plan-mapset handler must contain a try block");

    assert!(
        !before_try.contains("const provider"),
        "provider resolution must sit inside the try block so errors cannot reject unhandled"
    );
    assert!(
        inside_try.contains("currentImageryStyle") && inside_try.contains("'/api/mapset/plan'"),
        "try block must derive provider from currentImageryStyle before posting to /api/mapset/plan"
    );
}
