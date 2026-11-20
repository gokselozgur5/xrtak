use xrtak_core::cog::ingest_geotiff_dsm;

fn entry(tag: u16, ftype: u16, count: u32, val: u32) -> Vec<u8> {
    let mut e = tag.to_le_bytes().to_vec();
    e.extend_from_slice(&ftype.to_le_bytes());
    e.extend_from_slice(&count.to_le_bytes());
    e.extend_from_slice(&val.to_le_bytes());
    e
}

#[test]
fn deflate_compressed_strip_is_not_decoded_as_raw_samples() {
    let mut tiff = b"II\x2a\x00\x08\x00\x00\x00".to_vec();
    tiff.extend_from_slice(&6u16.to_le_bytes());
    tiff.extend(entry(256, 3, 1, 2));
    tiff.extend(entry(257, 3, 1, 2));
    tiff.extend(entry(258, 3, 1, 16));
    tiff.extend(entry(259, 3, 1, 8));
    tiff.extend(entry(273, 4, 1, 86));
    tiff.extend(entry(339, 3, 1, 2));
    tiff.extend_from_slice(&0u32.to_le_bytes());
    tiff.extend_from_slice(&[
        0x78, 0x9c, 0x4b, 0x61, 0xc8, 0x63, 0xa8, 0x60, 0x68, 0x62, 0x00, 0x00, 0x08, 0xa0, 0x01,
        0xcd,
    ]);
    let res = ingest_geotiff_dsm("dsm", 100.0, 2, &tiff);
    assert!(res.is_err(), "compressed data unexpectedly decoded as raw samples");
}
