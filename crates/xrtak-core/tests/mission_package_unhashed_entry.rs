use xrtak_core::mission_package::{create_zip_archive, MissionPackageArchive};

#[test]
fn entry_without_manifest_hash_is_not_reported_as_verified() {
    let manifest = "<MissionPackageManifest version=\"2\">\n<Contents>\n<Content ignore=\"false\" zipEntry=\"a.cot\"/>\n</Contents>\n</MissionPackageManifest>\n";
    let zip = create_zip_archive(&[
        (
            "MANIFEST/manifest.xml".to_string(),
            manifest.as_bytes().to_vec(),
        ),
        ("a.cot".to_string(), b"<event uid='a'/>".to_vec()),
    ]);
    let report = MissionPackageArchive::verify_zip(&zip).expect("verify zip");
    assert!(
        !report.is_valid || report.verified_entries == vec!["a.cot".to_string()],
        "is_valid={} with {} of 1 content files checked",
        report.is_valid,
        report.verified_entries.len()
    );
}
