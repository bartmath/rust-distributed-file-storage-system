use crate::internal::chunkserver_definition::ChunkserverInternal;
use async_trait::async_trait;
use quinn::{Endpoint, RecvStream, SendStream};
use storage_core::common::QuicServer;

#[async_trait]
impl QuicServer for ChunkserverInternal {
    fn listening_endpoint(&self) -> &Endpoint {
        self.internal_endpoint.as_ref()
    }

    async fn setup(&self) -> anyhow::Result<()> {
        let mut server_clone = self.clone();
        tokio::spawn(async move { server_clone.send_heartbeat().await });
        Ok(())
    }

    async fn handle_request(&self, send: SendStream, recv: RecvStream) -> anyhow::Result<()> {
        todo!(
            "currently chunkserver doesn't receive any messages from other servers, \
            in future add leases messages, chunks version checks, chunks forwarding in case of an error.",
        )
    }
}
