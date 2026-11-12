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
