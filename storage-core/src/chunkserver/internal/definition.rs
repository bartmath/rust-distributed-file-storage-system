use crate::chunk::{Chunk, ChunkId};
use crate::types::{Hostname, RackId, ServerId};
use arc_swap::ArcSwap;
use quinn::{Connection, Endpoint};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use storage_core::common::{
    ChunkServerDiscoverPayload, HeartbeatPayload, Message, MetadataServerInternalMessage,
};
use tokio::sync::Mutex;
use tokio::time::sleep;
use uuid::Uuid;

/// 'ChunkserverInternal' is a struct that is used for communication with 'MetadataServer' and other 'Chunkservers'
/// # Tasks include:
/// * sending stats to 'MetadataServer' via heartbeat
/// * ensuring consistency of the states of all chunk's replicas across different 'Chunkservers'
#[derive(Clone)]
pub struct ChunkserverInternal {
    server_id: ServerId,
    rack_id: RackId,

    heartbeat_interval: Duration,

    chunks: Arc<scc::HashMap<ChunkId, Chunk>>,

    pub(super) internal_endpoint: Arc<Endpoint>,

    metadata_server_addr: SocketAddr,
    metadata_server_hostname: Hostname,

    metadata_reconnect_lock: Arc<Mutex<()>>,
    metadata_server_connection: Arc<ArcSwap<Option<Connection>>>,
    chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
}

impl ChunkserverInternal {
    pub(crate) fn new(
        rack_id: RackId,
        chunks: Arc<scc::HashMap<ChunkId, Chunk>>,
        internal_endpoint: Arc<Endpoint>,
        metadata_server_addr: SocketAddr,
        metadata_server_hostname: Hostname,
        chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
    ) -> Self {
        ChunkserverInternal {
            server_id: Uuid::new_v4(),
            rack_id,
            heartbeat_interval: Duration::from_secs(180),
            chunks,
            internal_endpoint,
            metadata_server_addr,
            metadata_server_hostname,
            metadata_reconnect_lock: Arc::new(Mutex::new(())),
            metadata_server_connection: Arc::new(ArcSwap::from_pointee(None)),
            chunkserver_connections,
        }
    }

    async fn get_metadata_server_connection(&mut self) -> anyhow::Result<Connection> {
        let guard = self.metadata_server_connection.load();

        let Some(conn) = guard
            .as_ref()
            .as_ref()
            .filter(|c| c.close_reason().is_none())
        else {
            drop(guard);
            return self.reestablish_metadata_server_connection().await;
        };

        Ok(conn.clone())
    }
    async fn reestablish_metadata_server_connection(&self) -> anyhow::Result<Connection> {
        // We use Mutex to prevent many threads to simultaneously tring to create new connection
        // with the MetadataServer.
        let lock = self.metadata_reconnect_lock.lock().await;

        let conn = self.metadata_server_connection.load();
        let Some(conn) = conn
            .as_ref()
            .as_ref()
            .filter(|x| x.close_reason().is_none())
        else {
            let connecting = self
                .internal_endpoint
                .connect(self.metadata_server_addr, &self.metadata_server_hostname)?;
            let new_conn = connecting.await?;

            self.metadata_server_connection
                .store(Arc::new(Some(new_conn.clone())));

            drop(lock);

            self.metadata_server_handshake(new_conn.clone()).await?;

            return Ok(new_conn);
        };

        Ok(conn.clone())
    }

    async fn metadata_server_handshake(
        &self,
        metadata_server_conn: Connection,
    ) -> anyhow::Result<()> {
        let message =
            MetadataServerInternalMessage::ChunkServerDiscover(ChunkServerDiscoverPayload {
                rack_id: self.rack_id.clone(),
                server_id: self.server_id,
                stored_chunks: vec![],
            });

        let mut send_stream = metadata_server_conn.open_uni().await?;

        message.send(&mut send_stream).await

        // TODO: maybe the metadata server will return some answer
    }

    pub(super) async fn send_heartbeat(&mut self) -> anyhow::Result<()> {
        let conn = self.get_metadata_server_connection().await?;
        let (mut send, recv) = conn.open_bi().await?;

        loop {
            let message = MetadataServerInternalMessage::Heartbeat(HeartbeatPayload {
                server_id: self.server_id,
                active_client_connections: 1, // TODO: fetch those values in future
                available_space_bytes: 64 * 1024 * 1024,
            });
            message.send(&mut send).await?;
            sleep(self.heartbeat_interval).await;
        }
    }
}
