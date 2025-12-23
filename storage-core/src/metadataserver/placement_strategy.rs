use crate::types::ActiveChunkserver;
use rand::rng;
use rand::seq::IndexedRandom;
use std::net::SocketAddr;
use storage_core::common::ChunkserverLocation;

type ServerLocation = SocketAddr;

/// Selects chunkservers for new chunks.
///
/// # Arguments
/// * `n_chunks` - Number of chunks being placed.
/// * `candidates` - List of all available/healthy chunkservers.
/// * `replication_factor` - How many copies do we need? (e.g., 3)
///
/// # Returns
/// A vector of length 'n_chunks' of selected chunkservers.
pub(crate) trait PlacementStrategy {
    fn select_servers(
        &self,
        n_chunks: usize,
        active_chunkservers: &Vec<ActiveChunkserver>,
    ) -> Vec<ChunkserverLocation>;
}

#[derive(Debug, Clone)]
pub(crate) struct RandomPlacementStrategy {}

impl PlacementStrategy for RandomPlacementStrategy {
    fn select_servers(
        &self,
        n_chunks: usize,
        available_servers: &Vec<ActiveChunkserver>,
    ) -> Vec<ChunkserverLocation> {
        let mut rng = rng();

        (0..n_chunks)
            .map(|_| {
                available_servers
                    .choose(&mut rng)
                    .expect("No servers available yet")
            })
            .map(|chunkserver| {
                ChunkserverLocation::new(chunkserver.internal_address, chunkserver.hostname.clone())
            })
            .collect()
    }
}
