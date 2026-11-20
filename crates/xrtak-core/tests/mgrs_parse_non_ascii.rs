use xrtak_core::mgrs::MgrsCoord;

#[test]
fn non_ascii_square_identifier_is_an_error_not_a_panic() {
    let outcome = std::panic::catch_unwind(|| MgrsCoord::parse("31U\u{e9}").is_err());
    assert_eq!(outcome.ok(), Some(true), "expected Err, got a panic or Ok");
}
