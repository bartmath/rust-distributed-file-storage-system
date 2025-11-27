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

/// Sent by Chunkserver to MetadataServer as first message.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkServerDiscoverPayload {
    pub rack_id: Uuid,
}

/// Sent from MetadataServer to Chunkserver as a response to ChunkServerDiscoverPayload.
/// Contains new id of the Chunkserver which has been assigned by MetadataServer.
#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptNewChunkServerPayload {
    pub chunkserver_new_id: Uuid,
}

/// Sent regularly by ChunkServer to MetadataServer.
/// Contains all data and statistics required by MetadataServer
/// to make informed decision on chunks distribution between Chunkservers.
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatPayload {}

/// Sent by Client to MetadataServer.
/// Sends some data about the file to upload so that MetadataServer may decide
/// where to store file's chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkServersRequestPayload {}

/// Sent by MetadataServer to Client as a response to UploadChunkServersRequestPayload.
/// Contains list of Chunkservers (with their addresses) where the chunks have to be stored.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkServersResponsePayload {}

/// Sent from Client to Chunkserver.
/// Contains a Chunks to be stored on the Chunkserver.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkPayload {
    pub chunk_id: [u8; 32],
    pub checksum: [u8; 32],
    pub chunk_size: u64,
    pub session_token: Vec<u8>,
    pub chunk_token: Vec<u8>,
}

/// Sent from Client to MetadataServer.
/// Contains the file id, which Client wants to download.
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkserversRequestPayload {}

/// Sent from MetadataServer to Client as a response to DownloadChunkserversRequestPayload.
/// Contains list of Chunkservers (with their addresses) which store file's chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkserversResponsePayload {}

/// Sent from Client to ChunkServer.
/// Contains list of chunk ids which it wants to download from the ChunkServer.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksRequestPayload {}

/// Sent from Chunkserver to Client as a response to GetChunksRequestPayload.
/// Contains Chunks which have been requested by Client.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksResponsePayload {}

// TODO: SECOND PHASE

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderRequestMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteFileRequestMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesInFolderRequestMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesInFolderResponseMessage {}


storage_macros::register_types!{
    UploadChunkPayload,
    UploadChunkServersRequestPayload,
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de> + TypeId> Message for T {}