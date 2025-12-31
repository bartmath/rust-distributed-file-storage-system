use crate::types::{ActiveChunkserver, ChunkserverId, FailedChunkserver};
use quinn::{Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::{ChunkServerDiscoverPayload, HeartbeatPayload};

/// 'MetadataServerInternal' is a struct used for communication with chunkservers.
#[derive(Clone)]
pub struct MetadataServerInternal {
    pub(super) internal_endpoint: Arc<Endpoint>,

    active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
    failed_chunkservers: Arc<scc::HashIndex<ChunkserverId, FailedChunkserver>>,
}

impl MetadataServerInternal {
    pub(crate) fn new(
        internal_endpoint: Arc<Endpoint>,
        active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
        failed_chunkservers: Arc<scc::HashIndex<ChunkserverId, FailedChunkserver>>,
    ) -> Self {
        MetadataServerInternal {
            internal_endpoint,
            active_chunkservers,
            failed_chunkservers,
        }
    }

    pub(super) async fn discover_new_chunkserver(
        &self,
        send: &mut SendStream,
        payload: ChunkServerDiscoverPayload,
    ) -> anyhow::Result<()> {
        todo!("implement discovery new chunkserver")
    }

    pub(super) async fn accept_heartbeat(
        &self,
        send: &mut SendStream,
        payload: HeartbeatPayload,
    ) -> anyhow::Result<()> {
        todo!("implement accept heartbeat")
    }
}
