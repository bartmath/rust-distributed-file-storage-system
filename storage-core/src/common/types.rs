use crate::common::{ChunkserverMessage, Message, UploadChunkPayload};
use anyhow::bail;
use moka::future::Cache;
use quinn::{Connection, Endpoint};
use std::collections::HashMap;
use std::io::SeekFrom;
use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use uuid::Uuid;

type ChunkId = Uuid;
type ServerLocation = SocketAddr;
type ServerConnections = Cache<ServerLocation, Connection>;
type Hostname = String;

struct ClientState {
    endpoint: Endpoint,
    metadata_server_location: ServerLocation,
    metadata_server_connection: Option<Connection>,
    chunkserver_connections: ServerConnections,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkserverLocation {
    chunk_id: ChunkId,
    server_location: ServerLocation,
    server_hostname: Hostname,
}

impl ChunkserverLocation {
    pub fn new(server_location: ServerLocation, server_hostname: Hostname) -> Self {
        ChunkserverLocation {
            chunk_id: Uuid::new_v4(),
            server_location,
            server_hostname,
        }
    }

    fn with_metadata(self, file_path: String, offset: u64, chunk_size: u64) -> SendChunkMetadata {
        SendChunkMetadata {
            chunk_id: self.chunk_id,
            server_location: self.server_location,
            server_hostname: self.server_hostname,
            file_path,
            offset,
            chunk_size,
        }
    }
}

#[derive(Debug)]
struct SendChunkMetadata {
    chunk_id: ChunkId,
    server_location: ServerLocation,
    server_hostname: Hostname,
    offset: u64,
    chunk_size: u64,
    file_path: String,
}

impl SendChunkMetadata {
    async fn send(
        self,
        endpoint: Endpoint,
        connections: ServerConnections,
    ) -> anyhow::Result<ChunkId> {
        let conn = connections
            .try_get_with(self.server_location, async {
                self.connect_to_server(&endpoint).await
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

        if conn.close_reason().is_some() {
            connections.invalidate(&self.server_location).await;

            // TODO: right now we only try to reconnect one time. In future, the metadataserver
            // TODO: might return some number of backup chunkservers, which will be used for in case of errors.
            let new_conn = self.connect_to_server(&endpoint).await?;
            connections
                .insert(self.server_location, new_conn.clone())
                .await;

            return self.send_chunk(new_conn).await;
        }

        self.send_chunk(conn).await
    }

    async fn connect_to_server(&self, endpoint: &Endpoint) -> anyhow::Result<Connection> {
        let connecting = endpoint.connect(self.server_location, &self.server_hostname)?;
        let conn = connecting.await?;
        Ok(conn)
    }

    async fn send_chunk(self, conn: Connection) -> anyhow::Result<ChunkId> {
        let data = vec![0u8; 0];

        let payload = UploadChunkPayload {
            chunk_id: self.chunk_id,
            chunk_size: self.chunk_size,
            data,
        };

        let message = ChunkserverMessage::UploadChunk(payload);

        let mut file = File::open(self.file_path).await?;
        file.seek(SeekFrom::Start(self.offset)).await?;

        let (mut send, mut recv) = conn.open_bi().await?;

        message.send(&mut send).await?;
        let mut buf = vec![0u8; 64 * 1024]; // 64kB
        let mut sent = 0u64;

        while sent < self.chunk_size {
            let remaining = self.chunk_size - sent;
            let to_read = std::cmp::min(buf.len() as u64, remaining) as usize;

            let n = file.read(&mut buf[0..to_read]).await?;
            if n == 0 {
                bail!("Chunk read to few bytes");
            }

            send.write_all(&buf[0..n]).await?;

            sent += n as u64;
        }
        send.finish()?;

        Ok(self.chunk_id)
    }
}
