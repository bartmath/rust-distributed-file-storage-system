use std::net::SocketAddr;
use storage_core::common::ChunkServerDiscoverPayload;
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
    // Unique id of the primary server or None, if the primary isn't selected yet.
    pub(crate) primary: Option<ChunkserverId>,
    pub(crate) replicas: Vec<ChunkserverId>,
}

pub(crate) struct ActiveChunkserver {
    /// Unique server identifier.
    pub(crate) server_id: ChunkserverId,
    /// Rack id (TODO: use it in Placement Strategy)
    pub(crate) _rack_id: RackId,
    pub(crate) hostname: Hostname,
    /// Advertised address for internal communication with the chunkserver.
    pub(crate) _internal_address: SocketAddr,
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
            _rack_id: payload.rack_id.clone(),
            hostname: payload.hostname.clone(),
            _internal_address: payload.internal_address,
            external_address: payload.external_address,
            last_heartbeat: Instant::now(),
            client_request_count: 0,
            available_space: 0,
            chunks: payload.stored_chunks.clone(),
        }
    }
}
