use std::net::SocketAddr;
use tokio::time::Instant;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type FileId = String;
pub(crate) type ChunkserverId = Uuid;
pub(crate) type RackId = String;
pub(crate) type Hostname = String;

pub(crate) struct FileMetadata {
    chunks: Vec<ChunkId>,
}

pub(crate) struct ChunkMetadata {
    chunk_id: ChunkId,

    primary: ChunkserverId,
    replicas: Vec<ChunkserverId>,
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

    /// Chunks stored on the chunkserver.
    pub(crate) chunks: Vec<ChunkId>,
}

pub(crate) struct FailedChunkserver {
    /// Unique server identifier.
    pub(crate) server_id: ChunkserverId,
    pub(crate) rack_id: RackId,

    /// Chunks stored on the chunkserver.
    pub(crate) chunks: Vec<ChunkId>,
}
