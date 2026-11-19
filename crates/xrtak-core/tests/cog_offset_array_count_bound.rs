use std::sync::mpsc;
use std::time::Duration;
use xrtak_core::cog::parse_geotiff_metadata;

#[test]
fn strip_offsets_count_is_bounded_by_the_buffer_length() {
    let mut tiff = b"II\x2a\x00\x08\x00\x00\x00".to_vec();
    tiff.extend_from_slice(&8u16.to_le_bytes());
    for _ in 0..8 {
        tiff.extend_from_slice(&273u16.to_le_bytes()); // StripOffsets
        tiff.extend_from_slice(&4u16.to_le_bytes()); // LONG
        tiff.extend_from_slice(&u32::MAX.to_le_bytes()); // value count
        tiff.extend_from_slice(&0x7fff_fff0u32.to_le_bytes()); // array offset past buffer
    }
    tiff.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(tiff.len(), 110);

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || tx.send(parse_geotiff_metadata(&tiff).is_err()));
    let outcome = rx.recv_timeout(Duration::from_secs(5));
    assert_eq!(
        outcome,
        Ok(true),
        "a 110-byte file must be rejected promptly"
    );
}
