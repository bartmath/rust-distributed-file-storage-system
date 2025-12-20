use crate::chunk::Chunk;
use crate::external::ChunkserverExternal;
use anyhow::Result;
use quinn::{Connection, Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::{
    ClientMessage, DownloadChunkRequestPayload, DownloadChunkResponsePayload, FINAL_STORAGE_ROOT,
    Message, RequestStatusPayload, UploadChunkPayload,
};
use tokio::{fs, join};

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
        mut send: SendStream,
        payload: UploadChunkPayload,
    ) -> Result<()> {
        let chunk = Chunk {
            id: payload.chunk_id,
            size: payload.chunk_size,
        };

        if self
            .chunks
            .insert_async(payload.chunk_id, chunk)
            .await
            .is_err()
        {
            // File was already uploaded
            let _ = join!(
                fs::remove_file(&payload.data),
                ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest).send(&mut send)
            );

            return Ok(());
        }

        let chunk_final_path = FINAL_STORAGE_ROOT
            .get()
            .expect("Final storage path not initialized via config")
            .join(payload.chunk_id.to_string());

        fs::rename(&payload.data, &chunk_final_path).await?;

        ClientMessage::RequestStatus(RequestStatusPayload::Ok)
            .send(&mut send)
            .await?;

        Ok(())
    }

    pub(crate) async fn handle_download(
        &self,
        mut send: SendStream,
        payload: DownloadChunkRequestPayload,
    ) -> Result<()> {
        let chunk_size = self
            .chunks
            .read_async(&payload.chunk_id, |_, chunk| chunk.size)
            .await;

        let Some(chunk_size) = chunk_size else {
            // Chunk doesn't exist
            let _ = ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest)
                .send(&mut send)
                .await;

            return Ok(());
        };

        let chunk_path = FINAL_STORAGE_ROOT
            .get()
            .expect("Final storage path not initialized via config")
            .join(payload.chunk_id.to_string());

        let message = ClientMessage::DownloadChunkResponse(DownloadChunkResponsePayload {
            chunk_id: payload.chunk_id,
            chunk_size,
            offset: 0,
            data: chunk_path,
        });

        message.send(&mut send).await?;

        Ok(())
    }
}
