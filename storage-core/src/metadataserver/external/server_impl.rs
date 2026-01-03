use crate::external::MetadataServerExternal;
use async_trait::async_trait;
use quinn::{Endpoint, RecvStream, SendStream};
use storage_core::common::MetadataServerExternalMessage::{
    ChunkPlacementRequest, GetClientFolderStructureRequest, GetFilePlacementRequest,
    UpdateClientFolderStructure,
};
use storage_core::common::{
    Message, MessagePayload, MetadataServerExternalMessage, QuicServer, RequestStatusPayload,
};

#[async_trait]
impl QuicServer for MetadataServerExternal {
    fn listening_endpoint(&self) -> &Endpoint {
        &self.client_endpoint
    }

    async fn setup(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let res = match MetadataServerExternalMessage::recv(&mut recv).await? {
            ChunkPlacementRequest(payload) => self.place_file(&mut send, payload).await,
            GetFilePlacementRequest(payload) => self.fetch_file_placement(&mut send, payload).await,
            GetClientFolderStructureRequest(payload) => {
                self.fetch_folder_structure(&mut send, payload).await
            }
            UpdateClientFolderStructure(payload) => {
                self.update_folder_structure(&mut send, payload).await
            }
        };

        if res.is_err() {
            let _ = RequestStatusPayload::InternalServerError
                .send_payload(&mut send)
                .await;
        }

        Ok(())
    }
}
