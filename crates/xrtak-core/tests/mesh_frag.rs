use xrtak_core::mesh_frag::{fragment_payload, MeshReassembler};

#[test]
fn test_round_trip_single_fragment() {
    let mut reassembler = MeshReassembler::new(5000);
    let payload = b"Hello, MeshTAK!".to_vec();
    let fragments = fragment_payload(1, &payload, 100).unwrap();
    assert_eq!(fragments.len(), 1);

    let res = reassembler.accept(fragments[0].clone(), 1000);
    assert_eq!(res, Some(payload));
    assert!(reassembler.buffers.is_empty());
}

#[test]
fn test_round_trip_multi_fragment_in_order() {
    let mut reassembler = MeshReassembler::new(5000);
    let payload = vec![0xAB; 3500];
    let fragments = fragment_payload(42, &payload, 1000).unwrap();
    assert_eq!(fragments.len(), 4);

    for (i, frag) in fragments.iter().enumerate() {
        let res = reassembler.accept(frag.clone(), 1000 + (i as u64) * 10);
        if i < 3 {
            assert_eq!(res, None);
            assert_eq!(reassembler.buffers.len(), 1);
        } else {
            assert_eq!(res, Some(payload.clone()));
            assert!(reassembler.buffers.is_empty());
        }
    }
}
