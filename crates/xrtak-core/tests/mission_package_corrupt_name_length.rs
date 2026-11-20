use xrtak_core::mission_package::MissionPackageArchive;

#[test]
fn oversized_central_directory_name_length_is_an_error_not_a_panic() {
    let mut pkg = MissionPackageArchive::new("PKG-1", "Package");
    pkg.add_cot_file("a.cot", "<event uid='a'/>");
    let mut zip = pkg.build_zip().expect("build zip");

    // Corrupt the file name length of the first central directory record.
    let cd = zip
        .windows(4)
        .position(|w| w == [0x50, 0x4b, 0x01, 0x02])
        .expect("central directory record");
    zip[cd + 28] = 0xff;
    zip[cd + 29] = 0xff;

    let outcome = std::panic::catch_unwind(|| MissionPackageArchive::parse_zip(&zip).is_err());
    assert_eq!(outcome.ok(), Some(true), "expected Err, got a panic or Ok");
}
