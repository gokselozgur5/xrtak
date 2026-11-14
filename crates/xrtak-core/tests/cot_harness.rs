use xrtak_core::cot_harness::*;
use xrtak_core::GeoOrigin;

#[test]
fn test_cot_harness_happy_path_batch() {
    let mut harness = CotParseHarness::default();
    let xml1 = r#"<event uid="A-1" type="a-f-G">
        <point lat="39.92" lon="32.85" hae="950.0"/>
        <detail><contact callsign="VIPER"/></detail>
    </event>"#;
    let xml2 = r#"<event uid="B-2" type="a-h-A">
        <point lat="39.92" lon="32.85" hae="1200.0"/>
        <detail><contact callsign="BOGEY"/></detail>
    </event>"#;
    let batch = vec![
        RawCotMessage::new(xml1, 1000),
        RawCotMessage::new(xml2, 1001),
    ];
    let results = harness.parse_batch(&batch);
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert!(results[1].is_ok());
    assert_eq!(harness.metrics.total_processed, 2);
    assert_eq!(harness.metrics.total_success, 2);
    assert_eq!(harness.metrics.friendly_count, 1);
    assert_eq!(harness.metrics.hostile_count, 1);
}
