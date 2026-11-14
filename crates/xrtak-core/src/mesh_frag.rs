//! Peer-to-peer mesh packet fragmentation and reassembly.

use serde::{Deserialize, Serialize};

/// Header length in bytes for serialized wire packet.
pub const MESH_FRAGMENT_HEADER_SIZE: usize = 12;

/// Maximum payload chunk size supported per packet.
pub const MAX_FRAGMENT_PAYLOAD: usize = 1400;

/// Header identifying a fragment of a larger message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentHeader {
    pub message_id: u32,
    pub fragment_index: u16,
    pub total_fragments: u16,
    pub total_length: u32,
}
