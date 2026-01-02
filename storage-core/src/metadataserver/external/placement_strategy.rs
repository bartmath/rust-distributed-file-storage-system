use crate::types::{ActiveChunkserver, ChunkserverId};
use async_trait::async_trait;
use rand::rng;
use rand::seq::IndexedRandom;
use std::sync::Arc;
use storage_core::common::config::N_CHUNK_REPLICAS;
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
#[async_trait]
pub(crate) trait PlacementStrategy {
    async fn select_servers(
        &self,
        n_chunks: usize,
        active_chunkservers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,
    ) -> Vec<(PrimaryServerId, Vec<SecondaryServerId>)>;
}

#[derive(Debug, Clone)]
pub(crate) struct RandomPlacementStrategy {}

#[async_trait]
impl PlacementStrategy for RandomPlacementStrategy {
    async fn select_servers(
        &self,
        n_chunks: usize,
        available_servers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,
    ) -> Vec<(PrimaryServerId, Vec<SecondaryServerId>)> {
        let mut candidates = Vec::new();
        available_servers
            .iter_async(|k, _| {
                candidates.push(k.clone());
                true
            })
            .await;

        if candidates.len() < N_CHUNK_REPLICAS + 1 {
            return Vec::new();
        }

        let mut rng = rng();

        (0..n_chunks)
            .map(|_| {
                let mut selected: Vec<_> = candidates
                    .choose_multiple(&mut rng, N_CHUNK_REPLICAS + 1)
                    .copied()
                    .collect();

                // Already considered the case where too few servers were generated.
                let primary = selected.remove(0);

                (primary, selected)
            })
            .collect()
    }
}
