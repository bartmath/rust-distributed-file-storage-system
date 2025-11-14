use bincode;
use std::error::Error;
use std::net::SocketAddr;
use dashmap::DashMap;
use std::sync::Arc;
use quinn::{Connection, Endpoint, ServerConfig};
use uuid::Uuid;
use rust_fs::common::{ChunkServerDiscoverMessage, HeartbeatMessage};

struct Chunk {

}

struct ChunkServer {
    // The id of the chunkserver is assigned by metadataserver
    id: Option<Uuid>,
    rack_id: Uuid,
    chunks: Arc<DashMap<Uuid, Chunk>>,

    clients_endpoint: Endpoint,
    metadata_server_endpoint: Endpoint,

    client_connections: Arc<DashMap<Uuid, Connection>>,
    metadata_server_connection: Option<Arc<Connection>>,
}

impl ChunkServer {
    fn new(
        rack_id: Uuid,
        clients_config: ServerConfig,
        clients_endpoint_addr: SocketAddr,
        metadata_server_config: ServerConfig,
        metadata_endpoint_addr: SocketAddr,
    ) -> Result<Self, Box<dyn Error>> {
        let clients_endpoint = Endpoint::server(clients_config, clients_endpoint_addr)?;
        let metadata_server_endpoint = Endpoint::server(metadata_server_config, metadata_endpoint_addr)?;
        Ok(ChunkServer{
            id: None,
            rack_id,
            chunks: Arc::new(DashMap::new()),
            clients_endpoint,
            metadata_server_endpoint,
            client_connections: Arc::new(DashMap::new()),
            metadata_server_connection: None,
        })
    }

    async fn reestablish_metadata_server_connection(&self) -> anyhow::Result<()>  {
        Ok(())
    }

    async fn establish_metadata_server_connection(&self) -> anyhow::Result<()>  {
        self.reestablish_metadata_server_connection().await?;

        let message = ChunkServerDiscoverMessage {
            rack_id: self.rack_id
        };

        if let Some(conn) = self.metadata_server_connection.clone() {
            let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
            let bytes = bincode::serialize(&message)?;
            send_stream.write_all(&bytes).await?;

        }

        Ok(())
    }

    async fn send_heartbeat(self, metadata_server_addr: &str, id: &str) {
        loop {
            let hb = HeartbeatMessage {self.rack_id};
            send_to(metadata_server_addr, hb).await;
            sleep(heartbeat_interval).await;
        }
    }
}