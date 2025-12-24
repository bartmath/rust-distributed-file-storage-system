use crate::chunk::Chunk;
use crate::external::ChunkserverExternal;
use anyhow::Result;
use quinn::{Connection, Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::config::FINAL_STORAGE_ROOT;
use storage_core::common::{
    ClientMessage, DownloadChunkRequestPayload, DownloadChunkResponsePayload, Message,
    RequestStatusPayload, UploadChunkPayload,
};
use tokio::{fs, join};

impl ChunkserverExternal {
    pub(crate) fn new(
        chunks: Arc<scc::HashMap<crate::external::definition::ChunkId, Chunk>>,
        client_endpoint: Arc<Endpoint>,
        internal_endpoint: Arc<Endpoint>,
        chunkserver_connections: Arc<
            scc::HashMap<crate::external::definition::ServerId, Connection>,
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
        send: &mut SendStream,
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
                ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest).send(send)
            );

            return Ok(());
        }

        let chunk_final_path = FINAL_STORAGE_ROOT
            .get()
            .expect("Final storage path not initialized via config")
            .join(payload.chunk_id.to_string());

        fs::rename(&payload.data, &chunk_final_path).await?;

        ClientMessage::RequestStatus(RequestStatusPayload::Ok)
            .send(send)
            .await?;

        Ok(())
    }

    pub(crate) async fn handle_download(
        &self,
        send: &mut SendStream,
        payload: DownloadChunkRequestPayload,
    ) -> Result<()> {
        let chunk_size = self
            .chunks
            .read_async(&payload.chunk_id, |_, chunk| chunk.size)
            .await;

        let Some(chunk_size) = chunk_size else {
            // Chunk doesn't exist
            let _ = ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest)
                .send(send)
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

        message.send(send).await?;

        Ok(())
    }
}
