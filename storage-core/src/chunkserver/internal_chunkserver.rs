use crate::external_chunkserver::Chunk;
use anyhow::bail;
use async_trait::async_trait;
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use std::sync::Arc;
use storage_core::common::ChunkserverMessage::{
    AcceptNewChunkServer, DownloadChunkRequest, UploadChunk,
};
use storage_core::common::{ChunkserverMessage, Message, QuicServer};
use uuid::Uuid;

type Hostname = String;

type ServerId = Uuid;
type RackId = String;
type ChunkId = Uuid;

pub(crate) struct ChunkserverExternal {
    chunks: Arc<scc::HashMap<ChunkId, Chunk>>,

    client_endpoint: Arc<Endpoint>,
    internal_endpoint: Arc<Endpoint>,

    chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
}

impl ChunkserverExternal {
    pub(crate) fn new(
        chunks: Arc<scc::HashMap<ChunkId, Chunk>>,
        client_endpoint: Arc<Endpoint>,
        internal_endpoint: Arc<Endpoint>,
        chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
    ) -> Self {
        ChunkserverExternal {
            chunks,
            client_endpoint,
            internal_endpoint,
            chunkserver_connections,
        }
    }
}

impl Clone for ChunkserverExternal {
    fn clone(&self) -> Self {
        ChunkserverExternal {
            chunks: self.chunks.clone(),
            client_endpoint: self.client_endpoint.clone(),
            internal_endpoint: self.internal_endpoint.clone(),
            chunkserver_connections: self.chunkserver_connections.clone(),
        }
    }
}

#[async_trait]
impl QuicServer for ChunkserverExternal {
    fn listening_endpoint(&self) -> &Endpoint {
        self.client_endpoint.as_ref()
    }

    async fn handle_request(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        match ChunkserverMessage::recv(&mut recv).await? {
            UploadChunk(payload) => {
                // TODO: to make everything faster, we may consider omitting the save to Vec,
                // TODO: and stream the data to the file straight away
                // tokio::fs::write(payload.chunk_id.to_string(), payload.data).await?;
            }
            DownloadChunkRequest(payload) => {
                todo!("handle download request")
            }
            AcceptNewChunkServer(_) => bail!("Chunkserver was already accepted"),
        };

        Ok(())
    }
}
