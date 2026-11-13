use xrtak_core::{classify_cot_type, Affiliation, Dimension};

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

#[test]
fn test_classify_dimensions() {
    let cases = [
        ("a-f-A", Dimension::Air),
        ("a-f-G", Dimension::Ground),
        ("a-f-S", Dimension::Surface),
        ("a-f-U", Dimension::Unknown),
        ("a-f-Z", Dimension::Unknown),
        ("a-f", Dimension::Unknown),
    ];
    for (cot_type, expected_dim) in cases {
        let (_, dim) = classify_cot_type(cot_type);
        assert_eq!(dim, expected_dim, "mismatch for {cot_type}");
    }
}
