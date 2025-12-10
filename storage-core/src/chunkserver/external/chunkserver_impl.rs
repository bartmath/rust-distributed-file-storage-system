use crate::external::ChunkserverExternal;
use crate::internal::chunkserver_definition::Chunk;
use anyhow::Result;
use quinn::{Connection, Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::{DownloadChunkRequestPayload, FINAL_STORAGE_ROOT, UploadChunkPayload};
use tokio::fs;

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

impl ChunkserverExternal {
    pub(crate) fn new(
        chunks: Arc<scc::HashMap<crate::external::chunkserver_definition::ChunkId, Chunk>>,
        client_endpoint: Arc<Endpoint>,
        internal_endpoint: Arc<Endpoint>,
        chunkserver_connections: Arc<
            scc::HashMap<crate::external::chunkserver_definition::ServerId, Connection>,
        >,
    ) -> Self {
        ChunkserverExternal {
            chunks,
            client_endpoint,
            internal_endpoint,
            chunkserver_connections,
        }
    }

    pub(crate) async fn handle_upload(
        &self,
        send: SendStream,
        payload: UploadChunkPayload,
    ) -> Result<()> {
        let chunk_final_path = FINAL_STORAGE_ROOT
            .get()
            .expect("Final storage path not initialized via config")
            .join(payload.chunk_id.to_string());

        fs::rename(&payload.data, &chunk_final_path).await?;

        // TODO: send some response to client about the status of the send.

        Ok(())
    }

    pub(crate) async fn handle_download(
        &self,
        send: SendStream,
        payload: DownloadChunkRequestPayload,
    ) -> Result<()> {
        Ok(())
    }
}
