use crate::common::ChunkserverLocation;
use moka::future::Cache;
use quinn::Connection;
use std::net::SocketAddr;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type ServerLocation = SocketAddr;
pub(crate) type ServerConnections = Cache<ServerLocation, Connection>;
pub(crate) type Hostname = String;
pub type PrimaryLocation = ChunkserverLocation;
pub type SecondaryLocation = ChunkserverLocation;
