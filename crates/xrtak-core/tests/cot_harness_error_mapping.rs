use xrtak_core::cot_harness::{CotHarnessError, CotParseHarness, RawCotMessage};

#[test]
fn missing_uid_maps_to_missing_uid() {
    let mut harness = CotParseHarness::default();
    let raw = RawCotMessage::new(
        r#"<event type="a-f-G"><point lat="39.92" lon="32.85"/></event>"#,
        0,
    );
    let res = harness.parse_raw(&raw);
    assert!(matches!(res, Err(CotHarnessError::MissingUid)), "{res:?}");
}

#[test]
fn malformed_xml_with_point_present_maps_to_invalid_xml() {
    let mut harness = CotParseHarness::default();
    let raw = RawCotMessage::new(
        r#"<event uid="A" type="a-f-G"><point lat="1" lon="2"></event>"#,
        0,
    );
    let res = harness.parse_raw(&raw);
    assert!(
        matches!(res, Err(CotHarnessError::InvalidXml(_))),
        "{res:?}"
    );
}

#[test]
fn reach_missing_point_and_coordinate_range() {
    let mut harness = CotParseHarness::default();
    let raw_no_pt = RawCotMessage::new(r#"<event uid="A" type="a-f-G"></event>"#, 0);
    assert!(matches!(
        harness.parse_raw(&raw_no_pt),
        Err(CotHarnessError::MissingPoint)
    ));
    let raw_rng = RawCotMessage::new(
        r#"<event uid="A" type="a-f-G"><point lat="95.0" lon="32.85"/></event>"#,
        0,
    );
    assert!(matches!(
        harness.parse_raw(&raw_rng),
        Err(CotHarnessError::CoordinateRange(_))
    ));
}
