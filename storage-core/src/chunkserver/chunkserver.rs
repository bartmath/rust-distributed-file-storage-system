use crate::common::{ChunkServerDiscoverPayload, HeartbeatPayload, MAX_MESSAGE_SIZE, Message};

use bincode;
use dashmap::DashMap;
use quinn::{Connection, Endpoint, SendStream, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;
use crate::common;

struct Chunk {}

pub struct ChunkServer {
    // The id of the chunkserver is assigned by metadataserver
    id: Option<Uuid>,
    rack_id: Uuid,
    chunks: Arc<DashMap<Uuid, Chunk>>,
    heartbeat_interval: Duration,

    client_endpoint: Endpoint,
    internal_endpoint: Endpoint,

    metadata_server_addr: SocketAddr,

    client_connections: Arc<DashMap<Uuid, Connection>>,
    metadata_server_connection: Arc<RwLock<Option<Arc<Connection>>>>,
}

impl ChunkServer {
    pub fn new(
        rack_id: Uuid,
        clients_config: ServerConfig,
        clients_endpoint_addr: SocketAddr,
        heartbeat_interval: Duration,
        internal_connections_config: ServerConfig,
        internal_connections_addr: SocketAddr,
        metadata_server_addr: SocketAddr,
    ) -> Self {
        let clients_endpoint = Endpoint::server(clients_config, clients_endpoint_addr)
            .expect("Couldn't create client endpoint");
        let mut internal_endpoint =
            Endpoint::server(internal_connections_config, internal_connections_addr)
                .expect("Couldn't create internal endpoint");

        let path = std::env::current_dir().expect("Couldn't get current directory");
        let cert_path = path.join("../../../cert.der");
        let server_cert_der = std::fs::read(&cert_path).expect("Read");
        let mut roots = rustls::RootCertStore::empty();
        roots.add(CertificateDer::from(server_cert_der.as_ref())).expect("Hi");

        let mut client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        client_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

        let client_config =
            quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto).expect("Elo")));


        internal_endpoint.set_default_client_config(client_config);

        ChunkServer {
            id: None,
            rack_id,
            chunks: Arc::new(DashMap::new()),
            heartbeat_interval,
            client_endpoint: clients_endpoint,
            internal_endpoint,
            metadata_server_addr,
            client_connections: Arc::new(DashMap::new()),
            metadata_server_connection: Arc::new(RwLock::new(None)),
        }
    }

    async fn reestablish_metadata_server_connection(&mut self) -> anyhow::Result<()> {
        let connecting = self
            .internal_endpoint
            .connect(self.metadata_server_addr, "debug.localhost")?;
        let connection = connecting.await?;

        let mut guard = self.metadata_server_connection.write().await;
        *guard = Some(Arc::new(connection));

        Ok(())
    }

    pub async fn establish_metadata_server_connection(&mut self) -> anyhow::Result<()> {
        println!("Establishing metadata server connection");
        self.reestablish_metadata_server_connection().await?;
        println!("Connected");

        let message = ChunkServerDiscoverPayload {
            rack_id: self.rack_id,
        };

        let connection_guard = self.metadata_server_connection.read().await;

        /* if let Some(conn) = &*connection_guard {
            println!("Established metadata server connection - guard");
            let (mut send_stream, recv_stream) = conn.open_bi().await?;
            let bytes = bincode::serialize(&message)?;
            send_stream.write_all(&bytes).await?;
            println!("Message sent");

            match Message::from_stream(recv_stream, MAX_MESSAGE_SIZE).await? {
                Message::AcceptNewChunkServerMessage(payload) => {
                    self.id = Some(payload.chunkserver_new_id);
                }
                _ => {
                    panic!("Invalid response received for the ChunkServer discover message");
                }
            };

            self.send_heartbeat(send_stream).await?
        } */

        Ok(())
    }

    async fn send_heartbeat(&self, mut send_stream: SendStream) -> anyhow::Result<()> {
        loop {
            let message = HeartbeatPayload {};
            let bytes = bincode::serialize(&message)?;
            send_stream.write_all(&bytes).await?;
            println!("Sending Heartbeat message");
            sleep(self.heartbeat_interval).await;
        }
    }
}
