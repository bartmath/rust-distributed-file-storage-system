use moka::future::Cache;
use quinn::Connection;
use uuid::Uuid;

pub(crate) type ChunkId = Uuid;
pub(crate) type Hostname = String;
pub(crate) type ChunkserverId = Uuid;

pub(crate) type ChunkserverConnections = Cache<ChunkserverId, Connection>;
