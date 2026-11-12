use xrtak_core::cog::*;

fn entry(tag: u16, ty: u16, count: u32, val: u32) -> Vec<u8> {
    let mut e = Vec::new();
    e.extend_from_slice(&tag.to_le_bytes());
    e.extend_from_slice(&ty.to_le_bytes());
    e.extend_from_slice(&count.to_le_bytes());
    e.extend_from_slice(&val.to_le_bytes());
    e
}

#[test]
fn header_errors_are_reported() {
    assert!(parse_geotiff_metadata(b"II*").is_err());
    assert!(parse_geotiff_metadata(b"XX\x2a\x00\x08\x00\x00\x00").is_err());
    let truncated = parse_geotiff_metadata(b"II\x2a\x00\x40\x00\x00\x00");
    assert!(truncated.unwrap_err().contains("Truncated"));
}

fn int16_tiff(pixels: [i16; 4]) -> Vec<u8> {
    let mut b = b"II\x2a\x00\x08\x00\x00\x00".to_vec();
    b.extend_from_slice(&5u16.to_le_bytes());
    for e in [
        entry(256, 3, 1, 2),
        entry(257, 3, 1, 2),
        entry(258, 3, 1, 16),
        entry(273, 4, 1, 74),
        entry(339, 3, 1, 2),
    ] {
        b.extend(e);
    }
    b.extend_from_slice(&0u32.to_le_bytes());
    pixels
        .iter()
        .for_each(|p| b.extend_from_slice(&p.to_le_bytes()));
    b
}

#[test]
fn metadata_reads_dimensions_and_offsets() {
    let m = parse_geotiff_metadata(&int16_tiff([1, 2, 3, 4])).unwrap();
    assert_eq!((m.width, m.height, m.bits_per_sample), (2, 2, 16));
    assert_eq!((m.sample_format, m.data_offsets), (2, vec![74]));
    assert!(m.origin_lon_lat.is_none() && m.pixel_scale_xy.is_none());
}
