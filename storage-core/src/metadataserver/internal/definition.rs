use crate::types::{ActiveChunkserver, ChunkserverId};
use quinn::{Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::{ChunkServerDiscoverPayload, HeartbeatPayload};

/// 'MetadataServerInternal' is a struct used for communication with chunkservers.
#[derive(Clone)]
pub struct MetadataServerInternal {
    pub(super) internal_endpoint: Arc<Endpoint>,

    active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
}

impl MetadataServerInternal {
    pub(crate) fn new(
        internal_endpoint: Arc<Endpoint>,
        active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
    ) -> Self {
        MetadataServerInternal {
            internal_endpoint,
            active_chunkservers,
        }
    }

    pub(super) async fn discover_new_chunkserver(
        &self,
        send: &mut SendStream,
        payload: ChunkServerDiscoverPayload,
    ) -> anyhow::Result<()> {
        // TODO: for now we allow
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
        todo!("implement accept heartbeat")
    }

    pub(super) async fn prune_inactive_chunkservers(&self) {}
}
