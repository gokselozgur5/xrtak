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
fn dsm_ingestion_respects_int32_sample_format() {
    let mut b = b"II\x2a\x00\x08\x00\x00\x00".to_vec();
    b.extend_from_slice(&5u16.to_le_bytes());
    for e in [
        entry(256, 3, 1, 2),
        entry(257, 3, 1, 2),
        entry(258, 3, 1, 32),
        entry(273, 4, 1, 74),
        entry(339, 3, 1, 2),
    ] {
        b.extend(e);
    }
    b.extend_from_slice(&0u32.to_le_bytes());
    [910i32, 930, 850, 870]
        .iter()
        .for_each(|p| b.extend_from_slice(&p.to_le_bytes()));
    let r = ingest_geotiff_dsm("t", 100.0, 2, &b).unwrap();
    assert_eq!((r.min_asl_m, r.max_asl_m), (850.0, 930.0));
}

#[test]
fn raster_with_pixel_scale_derives_extent_from_ground_footprint() {
    let mut b = b"II\x2a\x00\x08\x00\x00\x00".to_vec();
    b.extend_from_slice(&7u16.to_le_bytes());
    for e in [
        entry(256, 3, 1, 2),
        entry(257, 3, 1, 2),
        entry(258, 3, 1, 16),
        entry(273, 4, 1, 172),
        entry(339, 3, 1, 2),
        entry(33550, 12, 3, 100),
        entry(33922, 12, 6, 124),
    ] {
        b.extend(e);
    }
    b.extend_from_slice(&0u32.to_le_bytes());
    while b.len() < 100 {
        b.push(0);
    }
    for v in [5000.0f64, 5000.0, 0.0] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for v in [0.0f64, 0.0, 0.0, 32.0, 39.0, 0.0] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for p in [100i16, 200, 300, 400] {
        b.extend_from_slice(&p.to_le_bytes());
    }
    let r = ingest_geotiff_dsm("ground-footprint", 1000.0, 2, &b).unwrap();
    assert_eq!(r.dem.extent_m, 5000.0);
}
