use xrtak_core::mission_manifest::{ManifestContentEntry, MissionPackageManifest};

#[test]
fn parse_xml_round_trips_version_and_content_parameters() {
    let mut manifest = MissionPackageManifest::new("UID-1", "Package");
    manifest.version = 3;
    manifest.contents.push(
        ManifestContentEntry::new("certs/client.p12", false).with_param("cert_alias", "UserCert"),
    );

    let parsed = MissionPackageManifest::parse_xml(&manifest.generate_xml()).expect("parse");
    assert_eq!(parsed.version, 3, "version read from the XML declaration");
    assert_eq!(
        parsed.contents[0].parameters,
        manifest.contents[0].parameters
    );
}

#[test]
fn parse_xml_rejects_non_manifest_input() {
    assert!(MissionPackageManifest::parse_xml("not a manifest").is_err());
}
