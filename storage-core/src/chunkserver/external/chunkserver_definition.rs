use crate::chunk::Chunk;
use quinn::{Connection, Endpoint};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) type Hostname = String;
pub(crate) type ServerId = Uuid;
pub(crate) type RackId = String;
pub(crate) type ChunkId = Uuid;

/// 'ChunkserverExternal' is a struct that is used for communication with clients.
pub struct ChunkserverExternal {
    pub(crate) chunks: Arc<scc::HashMap<ChunkId, Chunk>>,

    pub(crate) client_endpoint: Arc<Endpoint>,
    pub(crate) internal_endpoint: Arc<Endpoint>,

    pub(crate) chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
}
