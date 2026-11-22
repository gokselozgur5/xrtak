use xrtak_core::cog::ingest_geotiff_dsm;

fn entry(tag: u16, field_type: u16, count: u32, value: u32) -> Vec<u8> {
    let mut e = tag.to_le_bytes().to_vec();
    e.extend_from_slice(&field_type.to_le_bytes());
    e.extend_from_slice(&count.to_le_bytes());
    e.extend_from_slice(&value.to_le_bytes());
    e
}

#[test]
fn oversized_dimensions_are_rejected_without_panicking() {
    let mut tiff = b"II\x2a\x00\x08\x00\x00\x00".to_vec();
    tiff.extend_from_slice(&4u16.to_le_bytes());
    tiff.extend(entry(256, 4, 1, 0x8000_0000)); // ImageWidth
    tiff.extend(entry(257, 4, 1, 0x8000_0000)); // ImageLength
    tiff.extend(entry(258, 3, 1, 32)); // BitsPerSample
    tiff.extend(entry(273, 4, 1, 8)); // StripOffsets
    tiff.extend_from_slice(&0u32.to_le_bytes());

    let outcome =
        std::panic::catch_unwind(|| ingest_geotiff_dsm("dsm", 1500.0, 65, &tiff).is_err());
    assert_eq!(outcome.ok(), Some(true), "expected Err, got a panic or Ok");
}
