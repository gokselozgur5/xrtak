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

#[test]
fn test_round_trip_multi_fragment_out_of_order() {
    let mut reassembler = MeshReassembler::new(5000);
    let payload = b"The quick brown fox jumps over the lazy dog. 0123456789".to_vec();
    let fragments = fragment_payload(99, &payload, 10).unwrap();
    assert_eq!(fragments.len(), 6);

    let order = [3, 1, 4, 0, 5, 2];
    for (step, &idx) in order.iter().enumerate() {
        let res = reassembler.accept(fragments[idx].clone(), 1000 + (step as u64) * 10);
        if step < 5 {
            assert_eq!(res, None);
        } else {
            assert_eq!(res, Some(payload.clone()));
            assert!(reassembler.buffers.is_empty());
        }
    }
}

#[test]
fn test_duplicate_fragments_handled_gracefully() {
    let mut reassembler = MeshReassembler::new(5000);
    let payload = b"Duplicate test payload".to_vec();
    let fragments = fragment_payload(77, &payload, 10).unwrap();

    assert_eq!(reassembler.accept(fragments[0].clone(), 1000), None);
    assert_eq!(reassembler.accept(fragments[0].clone(), 1010), None);

    for frag in &fragments[1..fragments.len() - 1] {
        assert_eq!(reassembler.accept(frag.clone(), 1020), None);
    }
    assert_eq!(reassembler.accept(fragments[1].clone(), 1030), None);

    let res = reassembler.accept(fragments.last().unwrap().clone(), 1040);
    assert_eq!(res, Some(payload));
}
