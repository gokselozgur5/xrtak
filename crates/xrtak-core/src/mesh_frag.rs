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

/// Complete mesh fragment holding header and slice data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshFragment {
    pub header: FragmentHeader,
    pub payload: Vec<u8>,
}

impl FragmentHeader {
    pub fn to_bytes(&self) -> [u8; MESH_FRAGMENT_HEADER_SIZE] {
        let mut buf = [0u8; MESH_FRAGMENT_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.message_id.to_be_bytes());
        buf[4..6].copy_from_slice(&self.fragment_index.to_be_bytes());
        buf[6..8].copy_from_slice(&self.total_fragments.to_be_bytes());
        buf[8..12].copy_from_slice(&self.total_length.to_be_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < MESH_FRAGMENT_HEADER_SIZE {
            return None;
        }
        let message_id = u32::from_be_bytes(bytes[0..4].try_into().ok()?);
        let fragment_index = u16::from_be_bytes(bytes[4..6].try_into().ok()?);
        let total_fragments = u16::from_be_bytes(bytes[6..8].try_into().ok()?);
        let total_length = u32::from_be_bytes(bytes[8..12].try_into().ok()?);
        Some(Self {
            message_id,
            fragment_index,
            total_fragments,
            total_length,
        })
    }
}
