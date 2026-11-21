use xrtak_core::mesh_frag::{fragment_payload, MeshFragError};

#[test]
fn message_needing_more_than_u16_max_fragments_is_not_silently_corrupted() {
    let payload: Vec<u8> = vec![0u8; 70_000];

    // 70_000 one-byte fragments do not fit the u16 fragment counters.
    let result = fragment_payload(7, &payload, 1);
    assert!(matches!(
        result,
        Err(MeshFragError::TooManyFragments(70_000))
    ));
}
