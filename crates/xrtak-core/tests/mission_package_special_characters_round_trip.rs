use xrtak_core::mission_package::MissionPackageArchive;

#[test]
fn package_with_xml_special_characters_round_trips() {
    let mut pkg = MissionPackageArchive::new("PKG-1", "Op 'Spear' & Shield");
    pkg.add_cot_file("routes/R&D.cot", "<event uid='r1'/>");
    let zip = pkg.build_zip().expect("build zip");

    let report = MissionPackageArchive::verify_zip(&zip).expect("verify zip");
    assert!(report.is_valid, "untouched package failed: {:?}", report.failed_entries);

    let parsed = MissionPackageArchive::parse_zip(&zip).expect("parse zip");
    assert_eq!(parsed.name, "Op 'Spear' & Shield");
}
