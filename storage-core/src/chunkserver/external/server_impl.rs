use crate::external::ChunkserverExternal;
use async_trait::async_trait;
use quinn::{Endpoint, RecvStream, SendStream};
use storage_core::common::ChunkserverExternalMessage::{DownloadChunkRequest, UploadChunk};
use storage_core::common::{ChunkserverExternalMessage, Message, QuicServer};

#[async_trait]
impl QuicServer for ChunkserverExternal {
    fn listening_endpoint(&self) -> &Endpoint {
        self.client_endpoint.as_ref()
    }

    async fn setup(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        match ChunkserverExternalMessage::recv(&mut recv).await? {
            UploadChunk(payload) => self.handle_upload(send, payload).await?,
            DownloadChunkRequest(payload) => self.handle_download(send, payload).await?,
        };

        Ok(())
    }
}
