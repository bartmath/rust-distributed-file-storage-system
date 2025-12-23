use crate::external::MetadataServerExternal;
use crate::placement_strategy::RandomPlacementStrategy;
use crate::types::{
    ActiveChunkserver, ChunkId, ChunkMetadata, ChunkserverId, FailedChunkserver, FileId,
    FileMetadata,
};
use quinn::{Endpoint, SendStream};
use std::sync::Arc;
use storage_core::common::{
    ChunkPlacementRequestPayload, GetChunkPlacementRequestPayload,
    GetClientFolderStructureRequestPayload,
};

impl MetadataServerExternal {
    pub(crate) fn new(
        client_endpoint: Arc<Endpoint>,
        active_chunkservers: Arc<scc::HashIndex<ChunkserverId, ActiveChunkserver>>,
        failed_chunkservers: Arc<scc::HashIndex<ChunkserverId, FailedChunkserver>>,
        files: Arc<scc::HashMap<FileId, FileMetadata>>,
        chunks: Arc<scc::HashMap<ChunkId, ChunkMetadata>>,
    ) -> Self {
        MetadataServerExternal {
            client_endpoint,
            placement_strategy: RandomPlacementStrategy {},
            active_chunkservers,
            failed_chunkservers,
            files,
            chunks,
        }
    }

    pub(crate) async fn place_file(
        &self,
        send: SendStream,
        payload: ChunkPlacementRequestPayload,
    ) -> anyhow::Result<()> {
        todo!("unimplemented place_file")
    }

    pub(crate) async fn fetch_file_placement(
        &self,
        mut send: SendStream,
        payload: GetChunkPlacementRequestPayload,
    ) -> anyhow::Result<()> {
        todo!("unimplemented fetch_file_placement")
    }

    pub(crate) async fn fetch_folder_structure(
        &self,
        mut send: SendStream,
        payload: GetClientFolderStructureRequestPayload,
    ) -> anyhow::Result<()> {
        todo!("unimplemented fetch_folder_structure")
    }
}
