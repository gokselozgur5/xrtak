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

impl MeshFragment {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(MESH_FRAGMENT_HEADER_SIZE + self.payload.len());
        out.extend_from_slice(&self.header.to_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let header = FragmentHeader::from_bytes(bytes)?;
        let payload = bytes[MESH_FRAGMENT_HEADER_SIZE..].to_vec();
        Some(Self { header, payload })
    }
}

/// Splits payload into fragment chunks bounded by max_chunk_size.
pub fn fragment_payload(
    msg_id: u32,
    payload: &[u8],
    max_chunk: usize,
) -> Vec<MeshFragment> {
    let chunk_sz = max_chunk.clamp(1, MAX_FRAGMENT_PAYLOAD);
    if payload.is_empty() {
        return vec![MeshFragment {
            header: FragmentHeader {
                message_id: msg_id,
                fragment_index: 0,
                total_fragments: 1,
                total_length: 0,
            },
            payload: Vec::new(),
        }];
    }
    split_payload(msg_id, payload, chunk_sz)
}

fn split_payload(msg_id: u32, payload: &[u8], chunk_sz: usize) -> Vec<MeshFragment> {
    let chunks: Vec<&[u8]> = payload.chunks(chunk_sz).collect();
    let total = chunks.len() as u16;
    let len = payload.len() as u32;
    chunks
        .into_iter()
        .enumerate()
        .map(|(idx, chunk)| MeshFragment {
            header: FragmentHeader {
                message_id: msg_id,
                fragment_index: idx as u16,
                total_fragments: total,
                total_length: len,
            },
            payload: chunk.to_vec(),
        })
        .collect()
}

use std::collections::HashMap;

/// In-progress reassembly buffer for fragments belonging to one message.
#[derive(Debug, Clone)]
pub struct FragmentReassemblyBuffer {
    pub message_id: u32,
    pub total_fragments: u16,
    pub total_length: u32,
    pub received_fragments: HashMap<u16, Vec<u8>>,
    pub updated_unix_ms: u64,
}

impl FragmentReassemblyBuffer {
    pub fn new(header: &FragmentHeader, now_ms: u64) -> Self {
        Self {
            message_id: header.message_id,
            total_fragments: header.total_fragments,
            total_length: header.total_length,
            received_fragments: HashMap::new(),
            updated_unix_ms: now_ms,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.received_fragments.len() == self.total_fragments as usize
    }
}
