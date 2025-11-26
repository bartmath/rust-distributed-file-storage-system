use crate::common::{ChunkServerDiscoverPayload, HeartbeatPayload, MAX_MESSAGE_SIZE, Message};

use bincode;
use dashmap::DashMap;
use quinn::{Connection, Endpoint, SendStream, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

struct Chunk {}

pub struct ChunkServer {
    // The id of the chunkserver is assigned by metadataserver
    id: Option<Uuid>,
    rack_id: Uuid,
    chunks: Arc<DashMap<Uuid, Chunk>>,
    heartbeat_interval: Duration,

    client_endpoint: Endpoint,
    internal_endpoint: Endpoint,

    client_connections: Arc<DashMap<Uuid, Connection>>,
    metadata_server_connection: Option<Arc<Connection>>,
}

impl ChunkServer {
    pub fn new(
        rack_id: Uuid,
        clients_config: ServerConfig,
        clients_endpoint_addr: SocketAddr,
        heartbeat_interval: Duration,
        internal_connections_config: ServerConfig,
        internal_connections_addr: SocketAddr,
    ) -> Self {
        let clients_endpoint = Endpoint::server(clients_config, clients_endpoint_addr)
            .expect("Couldn't create client endpoint");
        let metadata_server_endpoint =
            Endpoint::server(internal_connections_config, internal_connections_addr)
                .expect("Couldn't create internal endpoint");

        ChunkServer {
            id: None,
            rack_id,
            chunks: Arc::new(DashMap::new()),
            heartbeat_interval,
            client_endpoint: clients_endpoint,
            internal_endpoint: metadata_server_endpoint,
            client_connections: Arc::new(DashMap::new()),
            metadata_server_connection: None,
        }
    }

    async fn reestablish_metadata_server_connection(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn establish_metadata_server_connection(&mut self) -> anyhow::Result<()> {
        self.reestablish_metadata_server_connection().await?;

        let message = ChunkServerDiscoverPayload {
            rack_id: self.rack_id,
        };

        if let Some(conn) = self.metadata_server_connection.clone() {
            let (mut send_stream, recv_stream) = conn.open_bi().await?;
            let bytes = bincode::serialize(&message)?;
            send_stream.write_all(&bytes).await?;

            match Message::from_stream(recv_stream, MAX_MESSAGE_SIZE).await? {
                Message::AcceptNewChunkServerMessage(payload) => {
                    self.id = Some(payload.chunkserver_new_id);
                }
                _ => {
                    panic!("Invalid response received for the ChunkServer discover message");
                }
            };

            self.send_heartbeat(send_stream).await?
        }

        Ok(())
    }

    async fn send_heartbeat(&self, mut send_stream: SendStream) -> anyhow::Result<()> {
        loop {
            let message = HeartbeatPayload {};
            let bytes = bincode::serialize(&message)?;
            send_stream.write_all(&bytes).await?;
            sleep(self.heartbeat_interval).await;
        }
    }
}
