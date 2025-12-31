use quinn::Chunk;
use serde::Serialize;
use std::net::SocketAddr;
use storage_core::common::ChunkServerDiscoverPayload;
use storage_core::common::config::N_CHUNK_REPLICAS;
use tokio::time::Instant;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type FileId = String;
pub(crate) type ChunkserverId = Uuid;
pub(crate) type RackId = String;
pub(crate) type Hostname = String;

pub(crate) struct FileMetadata {
    pub(crate) chunks: Vec<ChunkId>,
}

#[derive(Debug, Clone)]
pub(crate) struct ChunkMetadata {
    pub(crate) chunk_id: ChunkId,

    pub(crate) primary: ChunkserverId,
    pub(crate) replicas: Vec<ChunkserverId>,
}

pub(crate) struct ActiveChunkserver {
    /// Unique server identifier.
    pub(crate) server_id: ChunkserverId,
    pub(crate) rack_id: RackId,
    pub(crate) hostname: Hostname,
    /// Advertised address for internal communication with the chunkserver.
    pub(crate) internal_address: SocketAddr,
    /// Advertised address for external (client) communication with the chunkserver.
    pub(crate) external_address: SocketAddr,

    pub(crate) last_heartbeat: Instant,
    /// Number of client requests to the chunkserver in the period between two last heartbeats.
    pub(crate) client_request_count: u64,
    /// Available space on chunkserver's disk in bytes.
    pub(crate) available_space: u64,

    /// Chunks stored on the chunkserver.
    pub(crate) chunks: Vec<ChunkId>,
}

impl ActiveChunkserver {
    pub(crate) fn from_chunkserver_discover(payload: &ChunkServerDiscoverPayload) -> Self {
        ActiveChunkserver {
            server_id: payload.server_id,
            rack_id: payload.rack_id.clone(),
            hostname: payload.hostname.clone(),
            internal_address: payload.internal_address,
            external_address: payload.external_address,
            last_heartbeat: Instant::now(),
            client_request_count: 0,
            available_space: 0,
            chunks: vec![], // TODO: when changed from having [ChunkId, N_REPLICAS] to vec accept this and consider allowing for over-replication
        }
    }
}
