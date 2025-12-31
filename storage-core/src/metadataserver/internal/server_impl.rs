use crate::internal::MetadataServerInternal;
use async_trait::async_trait;
use quinn::{Endpoint, RecvStream, SendStream};
use storage_core::common::{Message, MetadataServerInternalMessage, QuicServer};

#[async_trait]
impl QuicServer for MetadataServerInternal {
    fn listening_endpoint(&self) -> &Endpoint {
        &self.internal_endpoint
    }

    async fn setup(&self) -> anyhow::Result<()> {
        let mut server_clone = self.clone();
        tokio::spawn(async move { server_clone.prune_inactive_chunkservers().await });
        Ok(())
    }

    async fn handle_request(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let _ = match MetadataServerInternalMessage::recv(&mut recv).await? {
            MetadataServerInternalMessage::ChunkServerDiscover(payload) => {
                self.discover_new_chunkserver(&mut send, payload).await
            }
            MetadataServerInternalMessage::Heartbeat(payload) => {
                self.accept_heartbeat(&mut send, payload).await
            }
        };

        Ok(())
    }
}
