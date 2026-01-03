use async_trait::async_trait;
use quinn::{RecvStream, SendStream};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait]
pub trait MessagePayload: Sized + Serialize + DeserializeOwned {
    // Context for receiving the payload
    type Ctx: Send + Sync;

    async fn send_payload(&self, send: &mut SendStream) -> anyhow::Result<()>;

    async fn recv_payload(recv: &mut RecvStream, _ctx: &Self::Ctx) -> anyhow::Result<Self>;
}

#[async_trait]
pub(crate) trait SerializablePayload:
    Sized + Serialize + DeserializeOwned + Send + Sync
{
    async fn send_payload(&self, send: &mut SendStream) -> anyhow::Result<()> {
        let bytes = bincode::serialize(&self)?;
        send.write_u32(bytes.len() as u32).await?;
        send.write_all(&bytes).await?;
        Ok(())
    }

    async fn recv_payload(recv: &mut RecvStream) -> anyhow::Result<Self> {
        let len = recv.read_u32().await?;
        let mut buffer = vec![0; len as usize];
        recv.read_exact(&mut buffer).await?;

        Ok(bincode::deserialize(&buffer)?)
    }
}

macro_rules! impl_serializable_payload {
    ($type:ty) => {
        impl crate::common::messages::payload::SerializablePayload for $type {}

        #[async_trait]
        impl crate::common::messages::payload::MessagePayload for $type {
            type Ctx = ();

            async fn send_payload(&self, send: &mut ::quinn::SendStream) -> ::anyhow::Result<()> {
                <Self as crate::common::messages::payload::SerializablePayload>::send_payload(
                    self, send,
                )
                .await
            }

            async fn recv_payload(
                recv: &mut ::quinn::RecvStream,
                _ctx: &(),
            ) -> ::anyhow::Result<Self> {
                <Self as crate::common::messages::payload::SerializablePayload>::recv_payload(recv)
                    .await
            }
        }
    };
}

#[async_trait]
pub trait ChunkPayload: Sized + Serialize + DeserializeOwned + Send + Sync {
    type Ctx: Send + Sync;

    async fn send_chunk(&self, send: &mut SendStream) -> anyhow::Result<()>;

    async fn recv_chunk(&mut self, recv: &mut RecvStream, _ctx: &Self::Ctx) -> anyhow::Result<()>;
}

#[async_trait]
impl<T: ChunkPayload> MessagePayload for T {
    type Ctx = T::Ctx;
    async fn send_payload(&self, send: &mut SendStream) -> anyhow::Result<()> {
        let metadata_bytes = bincode::serialize(&self)?;
        send.write_u32(metadata_bytes.len() as u32).await?;
        send.write_all(&metadata_bytes).await?;

        T::send_chunk(self, send).await?;

        Ok(())
    }

    async fn recv_payload(recv: &mut RecvStream, _ctx: &T::Ctx) -> anyhow::Result<Self> {
        let len = recv.read_u32().await?;

        let mut buffer = vec![0u8; len as usize];
        recv.read_exact(&mut buffer).await?;
        let mut payload: T = bincode::deserialize(&buffer)?;

        T::recv_chunk(&mut payload, recv, _ctx).await?;

        Ok(payload)
    }
}

pub(super) use impl_serializable_payload;
