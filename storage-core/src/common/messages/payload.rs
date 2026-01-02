use quinn::{RecvStream, SendStream};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) trait MessagePayload: Serialize + DeserializeOwned {
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
        let payload: Self = bincode::deserialize(&buffer)?;

        Ok(payload)
    }
}
