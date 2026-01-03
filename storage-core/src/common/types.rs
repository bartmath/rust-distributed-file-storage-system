use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type Hostname = String;
pub(crate) type ChunkserverId = Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkserverLocation {
    pub chunk_id: ChunkId,
    pub chunkserver_id: ChunkserverId,
    pub server_location: SocketAddr,
    pub server_hostname: Hostname,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkLocations {
    pub chunk_id: ChunkId,
    pub primary: ChunkserverLocation,
    pub replicas: Vec<ChunkserverLocation>,
}
