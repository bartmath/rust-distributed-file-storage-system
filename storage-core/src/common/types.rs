use crate::common::ChunkserverLocation;
use crate::common::config::N_CHUNK_REPLICAS;
use moka::future::Cache;
use quinn::Connection;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type ServerLocation = SocketAddr;
pub(crate) type ServerConnections = Cache<ServerLocation, Connection>;
pub(crate) type Hostname = String;
pub type PrimaryLocation = ChunkserverLocation;
pub type SecondaryLocation = ChunkserverLocation;
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkLocations {
    pub chunk_id: ChunkId,
    pub primary: PrimaryLocation,
    pub secondaries: [SecondaryLocation; N_CHUNK_REPLICAS], // TODO: change from returning a struct to having a Vec to account for possible failures
}
