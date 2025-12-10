use crate::common::types::{ChunkId, Hostname, ServerConnections, ServerLocation};
use crate::common::{ChunkserverExternalMessage, Message, UploadChunkPayload};
use quinn::{Connection, Endpoint};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
pub struct SendChunkMetadata {
    chunk_id: ChunkId,
    server_location: ServerLocation,
    server_hostname: Hostname,
    offset: u64,
    chunk_size: u64,
    file_path: PathBuf,
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
        let payload = UploadChunkPayload {
            chunk_id: self.chunk_id,
            chunk_size: self.chunk_size,
            offset: self.offset,
            data: self.file_path,
        };

        let message = ChunkserverExternalMessage::UploadChunk(payload);
        let (mut send, mut recv) = conn.open_bi().await?;

        message.send(&mut send).await?;
        send.finish()?;

        Ok(self.chunk_id)
    }
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

    fn with_file_path(self, file_path: PathBuf, chunk_size: u64) -> SendChunkMetadata {
        self.with_metadata(file_path, 0, chunk_size)
    }

    fn with_metadata(self, file_path: PathBuf, offset: u64, chunk_size: u64) -> SendChunkMetadata {
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
