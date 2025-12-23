use crate::external::MetadataServerExternal;
use async_trait::async_trait;
use quinn::{Endpoint, RecvStream, SendStream};
use storage_core::common::MetadataServerExternalMessage::{
    ChunkPlacementRequest, GetChunkPlacementRequest, GetClientFolderStructureRequest,
};
use storage_core::common::{Message, MetadataServerExternalMessage, QuicServer};

#[async_trait]
impl QuicServer for MetadataServerExternal {
    fn listening_endpoint(&self) -> &Endpoint {
        &self.client_endpoint
    }

    async fn setup(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn handle_request(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        match MetadataServerExternalMessage::recv(&mut recv).await? {
            ChunkPlacementRequest(payload) => self.place_file(send, payload).await?,
            GetChunkPlacementRequest(payload) => self.fetch_file_placement(send, payload).await?,
            GetClientFolderStructureRequest(payload) => {
                self.fetch_folder_structure(send, payload).await?
            }
        };

        Ok(())
    }
}
