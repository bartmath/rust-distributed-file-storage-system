use rand::rng;
use rand::seq::IndexedRandom;
use std::net::SocketAddr;
use crate::common::ChunkserverLocation;
use crate::metadataserver::metadataserver::ChunkserverMetadata;

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
        available_servers: &Vec<ChunkserverMetadata>,
    ) -> Vec<ChunkserverLocation>;
}

pub(crate) struct RandomPlacementStrategy {}

impl PlacementStrategy for RandomPlacementStrategy {
    fn select_servers(
        &self,
        n_chunks: usize,
        available_servers: &Vec<ChunkserverMetadata>,
    ) -> Vec<ChunkserverLocation> {
        let mut rng = rng();

        available_servers
            .choose_multiple(&mut rng, n_chunks)
            .map(|chunkserver| ChunkserverLocation::new(
                chunkserver.address,
                chunkserver.hostname.clone(),
            ))
            .collect()
    }
}
