use anyhow::Result;
use quinn::{RecvStream, SendStream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use storage_macros::ChunkPayload;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

pub(crate) trait MessagePayload: Serialize + DeserializeOwned {
    async fn send_payload(&self, send: &mut SendStream) -> Result<()> {
        let bytes = bincode::serialize(&self)?;
        send.write_u32(bytes.len() as u32).await?;
        send.write_all(&bytes).await?;
        Ok(())
    }

    async fn recv_payload(recv: &mut RecvStream) -> Result<Self> {
        let len = recv.read_u32().await?;
        let mut buffer = vec![0; len as usize];
        recv.read_exact(&mut buffer).await?;
        let payload: Self = bincode::deserialize(&buffer)?;

        Ok(payload)
    }
}

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
pub struct ChunkPlacementRequestPayload {}

/// Sent by MetadataServer to Client as a response to UploadChunkServersRequestPayload.
/// Contains list of Chunkservers (with their addresses) where the chunks have to be stored.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkPlacementResponsePayload {}

/// Sent from Client to Chunkserver.
/// Contains a Chunks to be stored on the Chunkserver.
#[derive(Serialize, Deserialize, Debug, ChunkPayload)]
pub struct UploadChunkPayload {
    pub chunk_id: [u8; 32],
    // pub checksum: [u8; 32],
    // pub chunk_size: u64,
    //pub session_token: Vec<u8>,
    // pub chunk_token: Vec<u8>,
    #[serde(skip)]
    pub data: Vec<u8>,
}

/// Sent from Client to MetadataServer.
/// Contains the file id, which Client wants to download.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunkPlacementRequestPayload {}

/// Sent from MetadataServer to Client as a response to DownloadChunkserversRequestPayload.
/// Contains list of Chunkservers (with their addresses) which store file's chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunkPlacementResponsePayload {}

/// Sent from Client to ChunkServer.
/// Contains list of chunk ids which it wants to download from the ChunkServer.
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkRequestPayload {}

/// Sent from Chunkserver to Client as a response to GetChunksRequestPayload.
/// Contains chunk which have been requested by Client.
#[derive(Serialize, Deserialize, Debug, ChunkPayload)]
pub struct DownloadChunkResponsePayload {
    #[serde(skip)]
    pub data: Vec<u8>,
}

// TODO: SECOND PHASE

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderRequestMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteFileRequestMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesInFolderRequestMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesInFolderResponseMessage {}

impl MessagePayload for AcceptNewChunkServerPayload {}
impl MessagePayload for ChunkServerDiscoverPayload {}
impl MessagePayload for HeartbeatPayload {}
impl MessagePayload for ChunkPlacementRequestPayload {}
impl MessagePayload for GetChunkPlacementRequestPayload {}
impl MessagePayload for ChunkPlacementResponsePayload {}
impl MessagePayload for GetChunkPlacementResponsePayload {}
impl MessagePayload for CreateFolderRequestMessage {}
impl MessagePayload for DeleteFileRequestMessage {}
impl MessagePayload for ListFilesInFolderRequestMessage {}
impl MessagePayload for ListFilesInFolderResponseMessage {}
impl MessagePayload for DownloadChunkRequestPayload {}
