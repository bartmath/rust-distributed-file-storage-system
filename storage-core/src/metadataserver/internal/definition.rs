use crate::types::{ActiveChunkserver, ChunkserverId, FailedChunkserver};
use quinn::Endpoint;
use std::sync::Arc;

/// 'MetadataServerInternal' is a struct used for communication with chunkservers.
#[derive(Clone)]
pub struct MetadataServerInternal {
    pub(crate) internal_endpoint: Arc<Endpoint>,

    pub(crate) active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
    pub(crate) failed_chunkservers: Arc<scc::HashIndex<ChunkserverId, FailedChunkserver>>,
}
