use crate::chunk::{Chunk, ChunkId};
use arc_swap::ArcSwap;
use quinn::{Connection, Endpoint};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

type Hostname = String;

type ServerId = Uuid;
type RackId = String;

/// 'ChunkserverInternal' is a struct that is used for communication with 'MetadataServer' and other 'Chunkservers'
/// # Tasks include:
/// * sending stats to 'MetadataServer' via heartbeat
/// * ensuring consistency of the states of all chunk's replicas across different 'Chunkservers'
pub struct ChunkserverInternal {
    pub(crate) server_id: ServerId,
    pub(crate) rack_id: RackId,

    pub(crate) heartbeat_interval: Duration,

    pub(crate) chunks: Arc<scc::HashMap<ChunkId, Chunk>>,

    pub(crate) internal_endpoint: Arc<Endpoint>,

    pub(crate) metadata_server_addr: SocketAddr,
    pub(crate) metadata_server_hostname: Hostname,

    pub(crate) metadata_reconnect_lock: Arc<Mutex<()>>,
    pub(crate) metadata_server_connection: Arc<ArcSwap<Option<Connection>>>,
    pub(crate) chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
}
