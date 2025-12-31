use crate::chunk::{Chunk, ChunkId};
use crate::types::{Hostname, RackId, ServerId};
use arc_swap::ArcSwap;
use quinn::{Connection, Endpoint};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use storage_core::common::config::{FINAL_STORAGE_ROOT, HEARTBEAT_INTERVAL};
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
    /// Unique identifier of the chunkserver.
    server_id: ServerId,
    /// Chunkserver's hostname for communication setup.
    hostname: Arc<Hostname>,
    rack_id: Arc<RackId>,
    /// Advertised address for internal (other chunkservers) communication with the chunkserver.
    internal_address: SocketAddr,
    /// Advertised address for external (client) communication with the chunkserver.
    external_address: SocketAddr,

    /// Counter of client requests since last heartbeat
    pub(super) requests_since_heartbeat: Arc<AtomicU64>,

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
        chunkserver_hostname: Hostname,
        rack_id: RackId,
        internal_address: SocketAddr,
        external_address: SocketAddr,
        requests_since_heartbeat: Arc<AtomicU64>,
        chunks: Arc<scc::HashMap<ChunkId, Chunk>>,
        internal_endpoint: Arc<Endpoint>,
        metadata_server_addr: SocketAddr,
        metadata_server_hostname: Hostname,
        chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
    ) -> Self {
        ChunkserverInternal {
            server_id: Uuid::new_v4(),
            hostname: Arc::new(chunkserver_hostname),
            rack_id: Arc::new(rack_id),
            internal_address,
            external_address,
            requests_since_heartbeat,
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
        // We use Mutex to prevent many threads to simultaneously trying to create new connection
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
        // TODO: change from removing chunks to keeping them
        self.chunks.retain_async(|_, _| false).await;

        let mut stored_chunks_ids = Vec::new();
        self.chunks
            .iter_async(|k, _| {
                stored_chunks_ids.push(k.clone());
                true
            })
            .await;

        let message =
            MetadataServerInternalMessage::ChunkServerDiscover(ChunkServerDiscoverPayload {
                server_id: self.server_id,
                hostname: self.hostname.to_string(),
                rack_id: self.rack_id.to_string(),
                internal_address: self.internal_address,
                external_address: self.external_address,
                stored_chunks: stored_chunks_ids,
            });

        let mut send_stream = metadata_server_conn.open_uni().await?;

        message.send(&mut send_stream).await

        // TODO: maybe the metadata server will return some answer
    }

    pub(super) async fn send_heartbeat(&mut self) -> anyhow::Result<()> {
        let conn = self.get_metadata_server_connection().await?;
        let (mut send, recv) = conn.open_bi().await?;

        loop {
            let client_requests_count = self.requests_since_heartbeat.swap(0, Ordering::Relaxed);
            let available_space = fs2::available_space(
                FINAL_STORAGE_ROOT
                    .get()
                    .expect("Final storage path not initialized via config"),
            )
            .unwrap_or(0);

            // We allow off up to 90% usage of the disk.
            let available_space = available_space * 9 / 10;

            let message = MetadataServerInternalMessage::Heartbeat(HeartbeatPayload {
                server_id: self.server_id,
                client_requests_count,
                available_space,
            });
            message.send(&mut send).await?;
            sleep(HEARTBEAT_INTERVAL).await;
        }
    }
}
