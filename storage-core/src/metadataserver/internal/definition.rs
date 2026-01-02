use crate::types::{ActiveChunkserver, ChunkId, ChunkMetadata, ChunkserverId};
use quinn::{Endpoint, SendStream};
use std::collections::HashSet;
use std::mem;
use std::sync::Arc;
use storage_core::common::config::{HEARTBEAT_INTERVAL, HEARTBEAT_MARGIN};
use storage_core::common::{ChunkServerDiscoverPayload, HeartbeatPayload};
use tokio::time::{Instant, sleep};

/// 'MetadataServerInternal' is a struct used for communication with chunkservers.
#[derive(Clone)]
pub struct MetadataServerInternal {
    pub(super) internal_endpoint: Arc<Endpoint>,

    active_chunkservers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,

    chunks: Arc<scc::HashMap<ChunkId, ChunkMetadata>>,
}

impl MetadataServerInternal {
    pub(crate) fn new(
        internal_endpoint: Arc<Endpoint>,
        active_chunkservers: Arc<scc::HashMap<ChunkserverId, ActiveChunkserver>>,
        chunks: Arc<scc::HashMap<ChunkId, ChunkMetadata>>,
    ) -> Self {
        MetadataServerInternal {
            internal_endpoint,
            active_chunkservers,
            chunks,
        }
    }

    pub(super) async fn discover_new_chunkserver(
        &self,
        send: &mut SendStream,
        payload: ChunkServerDiscoverPayload,
    ) -> anyhow::Result<()> {
        // TODO: check which chunks haven't been deleted yet and accept only those.
        // TODO: send a response with chunks the chunkserver has to delete - they're to old.
        let _ = self
            .active_chunkservers
            .insert_async(
                payload.server_id,
                ActiveChunkserver::from_chunkserver_discover(&payload),
            )
            .await;

        Ok(())
    }

    pub(super) async fn accept_heartbeat(
        &self,
        send: &mut SendStream,
        payload: HeartbeatPayload,
    ) -> anyhow::Result<()> {
        self.active_chunkservers
            .update_async(&payload.server_id, |_, server| {
                server.last_heartbeat = Instant::now();
                server.client_request_count = payload.client_requests_count;
                server.available_space = payload.available_space;
            })
            .await;

        Ok(())
    }

    pub(super) async fn prune_inactive_chunkservers(&self) {
        loop {
            let mut lost_chunk_replicas = Vec::new();
            self.active_chunkservers
                .retain_async(|_, server| {
                    if server.last_heartbeat + HEARTBEAT_INTERVAL + HEARTBEAT_MARGIN
                        >= Instant::now()
                    {
                        return true;
                    }

                    lost_chunk_replicas.push((server.server_id, mem::take(&mut server.chunks)));
                    false
                })
                .await;

            for (server_id, chunks) in lost_chunk_replicas.iter() {
                for chunk_id in chunks.iter() {
                    self.chunks.update_sync(chunk_id, |_, chunk_metadata| {
                        if chunk_metadata
                            .primary
                            .is_some_and(|primary| primary == *server_id)
                        {
                            chunk_metadata.primary = None;
                        }

                        chunk_metadata.replicas.retain(|&s_id| s_id != *server_id);
                    });
                }
            }

            let updated_chunks: HashSet<_> = lost_chunk_replicas
                .into_iter()
                .map(|(_, chunks)| chunks)
                .flatten()
                .collect();

            // TODO: replicate & select primary"

            sleep(HEARTBEAT_INTERVAL + HEARTBEAT_MARGIN).await;
        }
    }
}
