use xrtak_core::{classify_cot_type, Affiliation};

#[test]
fn test_classify_affiliations() {
    let cases = [
        ("a-f-G", Affiliation::Friendly),
        ("a-a-G", Affiliation::Friendly),
        ("a-h-G", Affiliation::Hostile),
        ("a-j-G", Affiliation::Hostile),
        ("a-k-G", Affiliation::Hostile),
        ("a-n-G", Affiliation::Neutral),
        ("a-u-G", Affiliation::Unknown),
        ("a-x-G", Affiliation::Unknown),
        ("unknown", Affiliation::Unknown),
    ];
    for (cot_type, expected_aff) in cases {
        let (aff, _) = classify_cot_type(cot_type);
        assert_eq!(aff, expected_aff, "mismatch for {cot_type}");
    }
}
