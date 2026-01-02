use crate::external::ChunkserverExternal;
use async_trait::async_trait;
use quinn::{Endpoint, RecvStream, SendStream};
use std::sync::atomic::Ordering;
use storage_core::common::ChunkserverExternalMessage::{DownloadChunkRequest, UploadChunk};
use storage_core::common::ClientMessage::RequestStatus;
use storage_core::common::{ChunkserverExternalMessage, Message, QuicServer, RequestStatusPayload};

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
        self.requests_since_heartbeat
            .fetch_add(1, Ordering::Relaxed);

        let res = match ChunkserverExternalMessage::recv(&mut recv).await? {
            UploadChunk(payload) => self.handle_upload(&mut send, payload).await,
            DownloadChunkRequest(payload) => self.handle_download(&mut send, payload).await,
        };

        if res.is_err() {
            let _ = RequestStatus(RequestStatusPayload::InternalServerError)
                .send(&mut send)
                .await;
        }

        Ok(())
    }
}
