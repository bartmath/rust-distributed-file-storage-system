use crate::types::{ActiveChunkserver, ChunkserverId};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_core::common::config::N_CHUNK_REPLICAS;

type ServerLocation = SocketAddr;
type PrimaryServerId = ChunkserverId;
type SecondaryServerId = ChunkserverId;

/// Selects chunkservers for new chunks.
///
/// # Arguments
/// * `n_chunks` - Number of chunks being placed.
/// * `active_chunkservers` - hashmap of active chunkservers.
///
/// # Returns
/// A vector of length 'n_chunks' of ids of primary and secondary servers for each chunk.
pub(crate) trait PlacementStrategy {
    fn select_servers(
        &self,
        n_chunks: usize,
        active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
    ) -> Vec<(PrimaryServerId, [SecondaryServerId; N_CHUNK_REPLICAS])>;
}

#[derive(Debug, Clone)]
pub(crate) struct RandomPlacementStrategy {}

impl PlacementStrategy for RandomPlacementStrategy {
    fn select_servers(
        &self,
        n_chunks: usize,
        available_servers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
    ) -> Vec<(PrimaryServerId, [SecondaryServerId; N_CHUNK_REPLICAS])> {
        todo!("implement the strategy")
    }
}
