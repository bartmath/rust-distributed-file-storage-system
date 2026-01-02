use crate::external::placement_strategy::{PlacementStrategy, RandomPlacementStrategy};
use crate::types::{
    ActiveChunkserver, ChunkId, ChunkMetadata, ChunkserverId, FileId, FileMetadata,
};
use anyhow::Context;
use futures::future::join_all;
use futures::{StreamExt, TryStreamExt, stream};
use quinn::{Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::config::{MAX_CHUNK_SIZE, MAX_SPAWNED_TASKS};
use storage_core::common::types::ChunkLocations;
use storage_core::common::{
    ChunkPlacementRequestPayload, ChunkPlacementResponsePayload, ChunkserverLocation,
    ClientMessage, GetClientFolderStructureRequestPayload, GetFilePlacementRequestPayload,
    GetFilePlacementResponsePayload, Message, RequestStatusPayload,
    UpdateClientFolderStructurePayload,
};
use uuid::Uuid;

/// 'MetadataServerExternal' is a struct used for communication with clients.
#[derive(Clone)]
pub struct MetadataServerExternal {
    pub(super) client_endpoint: Arc<Endpoint>,

    placement_strategy: RandomPlacementStrategy,

    active_chunkservers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,

    files: Arc<scc::HashMap<FileId, FileMetadata>>,
    chunks: Arc<scc::HashMap<ChunkId, ChunkMetadata>>,
}

impl MetadataServerExternal {
    pub(crate) fn new(
        client_endpoint: Arc<Endpoint>,
        active_chunkservers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,
        chunks: Arc<scc::HashMap<ChunkId, ChunkMetadata>>,
    ) -> Self {
        MetadataServerExternal {
            client_endpoint,
            placement_strategy: RandomPlacementStrategy {},
            active_chunkservers,
            files: Arc::new(scc::HashMap::new()),
            chunks,
        }
    }

    async fn resolve_chunk_locations(
        active_chunkservers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,
        chunk_id: ChunkId,
        primary: ChunkserverId,
        replicas: Vec<ChunkserverId>,
    ) -> anyhow::Result<ChunkLocations> {
        let to_location = move |s_id: ChunkserverId| {
            let active_chunkservers = active_chunkservers.clone();

            async move {
                active_chunkservers
                    .get_async(&s_id)
                    .await
                    .map(|server_entry| ChunkserverLocation {
                        chunk_id: s_id,
                        server_location: server_entry.get().external_address,
                        server_hostname: server_entry.get().hostname.clone(),
                    })
            }
        };

        let to_location_for_batch = to_location.clone();
        let to_locations = move |s_ids: Vec<ChunkserverId>| async move {
            join_all(s_ids.iter().map(|&id| to_location_for_batch(id)))
                .await
                .into_iter()
                .flatten()
                .collect()
        };

        Ok(ChunkLocations {
            chunk_id,
            primary: to_location(primary).await.context("Primary not found")?,
            replicas: to_locations(replicas).await,
        })
    }

    pub(super) async fn place_file(
        &self,
        send: &mut SendStream,
        payload: ChunkPlacementRequestPayload,
    ) -> anyhow::Result<()> {
        let n_chunks = payload.file_size.div_ceil(MAX_CHUNK_SIZE);
        let chunk_ids: Vec<_> = (0..n_chunks).map(|_| Uuid::new_v4()).collect();

        if self
            .files
            .insert_async(
                payload.filename,
                FileMetadata {
                    chunks: chunk_ids.clone(),
                },
            )
            .await
            .is_err()
        {
            // Prevent from creating the same file again (TODO: for given user).

            let _ = ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest)
                .send(send)
                .await;
            return Ok(());
        }

        let selected_servers_ids = self
            .placement_strategy
            .select_servers(n_chunks, self.active_chunkservers.clone())
            .await;

        let chunk_server_matchings: Vec<_> = chunk_ids
            .iter()
            .copied()
            .zip(selected_servers_ids)
            .collect();

        for (chunk_id, (primary, secondaries)) in &chunk_server_matchings {
            let _ = self
                .chunks
                .insert_async(
                    *chunk_id,
                    ChunkMetadata {
                        chunk_id: *chunk_id,
                        primary: Some(primary.clone()),
                        replicas: secondaries.clone(),
                    },
                )
                .await;
        }

        let active_chunkservers = self.active_chunkservers.clone();
        let selected_chunkservers = stream::iter(chunk_server_matchings)
            .map(|(chunk_id, (primary, secondaries))| {
                Self::resolve_chunk_locations(
                    active_chunkservers.clone(),
                    chunk_id,
                    primary,
                    secondaries,
                )
            })
            .buffer_unordered(MAX_SPAWNED_TASKS)
            .try_collect()
            .await?;

        ClientMessage::ChunkPlacementResponse(ChunkPlacementResponsePayload {
            selected_chunkservers,
        })
        .send(send)
        .await?;

        Ok(())
    }

    pub(super) async fn fetch_file_placement(
        &self,
        send: &mut SendStream,
        payload: GetFilePlacementRequestPayload,
    ) -> anyhow::Result<()> {
        let Some(file_chunks_ids) = self
            .files
            .read_async(&payload.filename, |_, file| file.chunks.clone())
            .await
        else {
            let _ = ClientMessage::RequestStatus(RequestStatusPayload::InvalidRequest)
                .send(send)
                .await;
            return Ok(());
        };

        let active_chunkservers_handle = self.active_chunkservers.clone();
        let chunks_handle = self.chunks.clone();
        let chunks_locations = stream::iter(file_chunks_ids)
            .map(move |chunk_id| {
                let chunks = chunks_handle.clone();
                let active_chunkservers = active_chunkservers_handle.clone();

                async move {
                    let chunk = chunks
                        .read_async(&chunk_id, |_, chunk| chunk.clone())
                        .await
                        .ok_or_else(|| {
                            anyhow::anyhow!("Chunk {} missing from metadata", chunk_id)
                        })?;

                    let Some(chunk_primary) = chunk.primary else {
                        return Err(anyhow::anyhow!(
                            "Chunk {} hasn't elected primary server",
                            chunk_id
                        ));
                    };

                    Self::resolve_chunk_locations(
                        active_chunkservers,
                        chunk_id,
                        chunk_primary,
                        chunk.replicas,
                    )
                    .await
                }
            })
            .buffer_unordered(MAX_SPAWNED_TASKS)
            .try_collect::<Vec<_>>()
            .await?;

        ClientMessage::GetFilePlacementResponse(GetFilePlacementResponsePayload {
            chunks_locations,
        })
        .send(send)
        .await?;

        Ok(())
    }

    pub(super) async fn fetch_folder_structure(
        &self,
        _send: &mut SendStream,
        _payload: GetClientFolderStructureRequestPayload,
    ) -> anyhow::Result<()> {
        todo!("unimplemented fetch_folder_structure")
    }

    pub(super) async fn update_folder_structure(
        &self,
        _send: &mut SendStream,
        _payload: UpdateClientFolderStructurePayload,
    ) -> anyhow::Result<()> {
        todo!("unimplemented update_folder_structure")
    }
}
