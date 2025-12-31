use crate::chunk::Chunk;
use crate::types::{ChunkId, ServerId};
use quinn::{Connection, Endpoint, SendStream};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use storage_core::common::config::FINAL_STORAGE_ROOT;
use storage_core::common::{
    ChunkTransfer, ClientMessage, DownloadChunkRequestPayload, DownloadChunkResponsePayload,
    Message, RequestStatusPayload, UploadChunkPayload,
};
use tokio::{fs, join};

/// 'ChunkserverExternal' is a struct used for communication with clients.
#[derive(Clone)]
pub struct ChunkserverExternal {
    chunks: Arc<scc::HashMap<ChunkId, Chunk>>,

    /// Counter of client requests since last heartbeat
    pub(super) requests_since_heartbeat: Arc<AtomicU64>,

    pub(super) client_endpoint: Arc<Endpoint>,
    internal_endpoint: Arc<Endpoint>,

    chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
}

impl ChunkserverExternal {
    pub(crate) fn new(
        chunks: Arc<scc::HashMap<ChunkId, Chunk>>,
        requests_since_heartbeat: Arc<AtomicU64>,
        client_endpoint: Arc<Endpoint>,
        internal_endpoint: Arc<Endpoint>,
        chunkserver_connections: Arc<scc::HashMap<ServerId, Connection>>,
    ) -> Self {
        ChunkserverExternal {
            chunks,
            requests_since_heartbeat,
            client_endpoint,
            internal_endpoint,
            chunkserver_connections,
        }
    }

    pub(super) async fn handle_upload(
        &self,
        send: &mut SendStream,
        payload: UploadChunkPayload,
    ) -> anyhow::Result<()> {
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
                fs::remove_file(&payload.chunk_transfer.data),
                ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest).send(send)
            );

            return Ok(());
        }

        let chunk_final_path = FINAL_STORAGE_ROOT
            .get()
            .expect("Final storage path not initialized via config")
            .join(payload.chunk_id.to_string());

        fs::rename(&payload.chunk_transfer.data, &chunk_final_path).await?;

        ClientMessage::RequestStatus(RequestStatusPayload::Ok)
            .send(send)
            .await?;

        Ok(())
    }

    pub(super) async fn handle_download(
        &self,
        send: &mut SendStream,
        payload: DownloadChunkRequestPayload,
    ) -> anyhow::Result<()> {
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

        ClientMessage::DownloadChunkResponse(DownloadChunkResponsePayload {
            chunk_id: payload.chunk_id,
            chunk_size,
            chunk_transfer: ChunkTransfer {
                offset: None,
                data: chunk_path,
            },
        })
        .send(send)
        .await?;

        Ok(())
    }
}
