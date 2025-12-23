use crate::placement_strategy::RandomPlacementStrategy;
use crate::types::{
    ActiveChunkserver, ChunkId, ChunkMetadata, ChunkserverId, FailedChunkserver, FileId,
    FileMetadata,
};
use quinn::Endpoint;
use std::sync::Arc;

/// 'MetadataServerExternal' is a struct used for communication with clients.
#[derive(Clone)]
pub struct MetadataServerExternal {
    pub(crate) client_endpoint: Arc<Endpoint>,

    pub(crate) placement_strategy: RandomPlacementStrategy,

    pub(crate) active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
    pub(crate) failed_chunkservers: Arc<scc::HashIndex<ChunkserverId, FailedChunkserver>>,

    pub(crate) files: Arc<scc::HashMap<FileId, FileMetadata>>,
    pub(crate) chunks: Arc<scc::HashMap<ChunkId, ChunkMetadata>>,
}
