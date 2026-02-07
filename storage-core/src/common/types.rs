use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type RackId = String;
pub(crate) type Hostname = String;
pub(crate) type ChunkserverId = Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkserverLocation {
    pub id: ChunkserverId,
    pub addr: SocketAddr,
    pub hostname: Hostname,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkLocations {
    pub chunk_id: ChunkId,
    pub primary: ChunkserverLocation,
    pub replicas: Vec<ChunkserverLocation>,
}
