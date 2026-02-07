use crate::chunkserver_connection_pool::ChunkserverConnectionPool;
use crate::types::ChunkId;
use futures::{StreamExt, stream};
use std::path::PathBuf;
use storage_core::common::config::{MAX_CHUNK_SIZE, MAX_SPAWNED_TASKS};
use storage_core::common::types::{ChunkLocations, ChunkserverLocation};
use storage_core::common::{
    ChunkTransfer, ChunkserverExternalMessage, Message, UploadChunkPayload,
};

pub(super) struct ClientChunkUploader {
    chunkserver_connection_pool: ChunkserverConnectionPool,
}

impl ClientChunkUploader {
    pub(super) fn new(
        chunkserver_connection_pool: ChunkserverConnectionPool,
    ) -> ClientChunkUploader {
        ClientChunkUploader {
            chunkserver_connection_pool,
        }
    }
}

impl ClientChunkUploader {
    async fn upload_chunk(
        offset: u64,
        file_path: PathBuf,
        chunk_id: ChunkId,
        chunk_size: u64,
        chunkserver: ChunkserverLocation,
        chunkserver_connection_pool: ChunkserverConnectionPool,
    ) -> anyhow::Result<()> {
        let (mut send, _) = chunkserver_connection_pool
            .get_chunkserver_stream(chunkserver)
            .await?;

        let chunk_transfer = ChunkTransfer::new(Some(offset), file_path, false);

        ChunkserverExternalMessage::UploadChunk(UploadChunkPayload {
            chunk_id,
            chunk_size,
            chunk_transfer,
        })
        .send(&mut send)
        .await?;

        send.finish()?;
        Ok(())
    }

    pub(super) async fn batch_upload_chunks<F>(
        &self,
        file_path: PathBuf,
        file_size: u64,
        chunk_locations: Vec<ChunkLocations>,
        mut chunkserver_extractor: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(ChunkLocations) -> Option<(ChunkId, ChunkserverLocation)>,
    {
        let mut upload_tasks = stream::iter(chunk_locations.into_iter().enumerate().filter_map(
            |(id, chunk_location)| {
                let file_path = file_path.clone();
                let offset = (MAX_CHUNK_SIZE * id) as u64;
                let chunk_size = std::cmp::min(file_size - offset, MAX_CHUNK_SIZE as u64);
                let (chunk_id, chunkserver) = chunkserver_extractor(chunk_location)?;
                let chunkserver_connection_pool = self.chunkserver_connection_pool.clone();

                Some(tokio::spawn(async move {
                    Self::upload_chunk(
                        offset,
                        file_path,
                        chunk_id,
                        chunk_size,
                        chunkserver,
                        chunkserver_connection_pool,
                    )
                    .await
                }))
            },
        ))
        .buffer_unordered(MAX_SPAWNED_TASKS);

        // Process results as they finish
        while let Some(result) = upload_tasks.next().await {
            result??;
        }

        Ok(())
    }
}
