use std::io::Write;
use bincode::Options;
use bytes::{BufMut, Bytes};
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait TypeId {
    const ID: u8;
}

pub trait Message: TypeId {}  // empty for now

pub const MAX_MESSAGE_SIZE: usize = 1024;

/*impl Message {
    pub async fn from_stream(
        mut stream: RecvStream,
        buffer_size: usize,
    ) -> anyhow::Result<Message> {
        let buffer = stream.read_to_end(buffer_size).await?;
        Ok(bincode::deserialize::<Message>(&buffer)?)
    }
}

impl Message {
    /// Send ANY message (small or metadata-only) over QUIC stream
    pub async fn send(&self, stream: &mut SendStream) -> anyhow::Result<()> {
        // Serialize metadata only
        let serialized = bincode::serialize(self)?;

        // Encode 4-byte LE length prefix
        let len_prefix = (serialized.len() as u32).to_le_bytes();

        // Send length prefix + serialized metadata
        stream.write_all(&len_prefix).await?;
        stream.write_all(&serialized).await?;
        stream.flush().await?;

        Ok(())
    }

    /*
    /// Receive ANY message (metadata only). Returns chunk_size if UploadChunksMessage
    pub async fn recv(stream: &mut RecvStream) -> Result<(Self, Option<u64>)> {
        // Read total frame length (4 bytes)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let total_len = u32::from_le_bytes(len_buf) as usize;

        // Read exact payload
        let mut payload = BytesMut::with_capacity(total_len);
        payload.resize(total_len, 0);
        stream.read_exact(&mut payload).await?;

        // Deserialize message
        let options = bincode::config::standard();
        let msg: Message = options.deserialize(&payload)?;

        // Return chunk_size if this is upload message
        let chunk_size = if let Message::UploadChunksMessage(meta) = &msg {
            Some(meta.chunk_size)
        } else {
            None
        };

        Ok((msg, chunk_size))
    } */
} */


storage_macros::register_types!{
    UploadChunkPayload,
    UploadChunkServersRequestPayload,
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de> + TypeId> Message for T {}