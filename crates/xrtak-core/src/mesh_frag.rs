//! Peer-to-peer mesh packet fragmentation and reassembly.

use serde::{Deserialize, Serialize};

/// Header length in bytes for serialized wire packet.
pub const MESH_FRAGMENT_HEADER_SIZE: usize = 12;

/// Maximum payload chunk size supported per packet.
pub const MAX_FRAGMENT_PAYLOAD: usize = 1400;
