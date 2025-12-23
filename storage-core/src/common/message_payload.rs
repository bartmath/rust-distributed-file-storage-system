use crate::common::chunk_send::ChunkserverLocation;
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use storage_macros::ChunkPayload;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

type ChunkId = Uuid;
type RackId = String;

pub static TMP_STORAGE_ROOT: OnceLock<PathBuf> = OnceLock::new();
pub static FINAL_STORAGE_ROOT: OnceLock<PathBuf> = OnceLock::new();

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

/// Sent by Chunkserver to MetadataServer as first message (or after reconnection with the MetadataServer).
/// Contains introduction of the chunkserver, with data that make it identifiable
/// and chunks that the Chunkserver stores.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkServerDiscoverPayload {
    pub server_id: Uuid,
    pub rack_id: RackId,
    pub stored_chunks: Vec<ChunkId>, // we will probably need to send also some other info about the chunk, not only the id
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
pub struct HeartbeatPayload {
    pub server_id: Uuid,
    pub active_client_connections: u32,
    pub available_space_bytes: u64,
}

/// Sent by Client to MetadataServer.
/// Sends some data about the file to upload so that MetadataServer may decide
/// where to store file's chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkPlacementRequestPayload {
    pub filename: String,
    pub file_size: usize,
}

/// Sent by MetadataServer to Client as a response to UploadChunkServersRequestPayload.
/// Contains list of Chunkservers (with their addresses) where the chunks have to be stored.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkPlacementResponsePayload {
    pub selected_chunkservers: Vec<ChunkserverLocation>,
}

/// Sent from Client to Chunkserver.
/// Contains a Chunks to be stored on the Chunkserver.
#[derive(Serialize, Deserialize, Debug, ChunkPayload)]
pub struct UploadChunkPayload {
    pub chunk_id: ChunkId,
    pub chunk_size: u64,
    // pub checksum: [u8; 32],
    #[serde(skip)]
    pub offset: u64, // helper for reconstructing the file
    #[serde(skip)]
    pub data: PathBuf,
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
pub struct DownloadChunkRequestPayload {
    pub chunk_id: ChunkId,
}

/// Sent from Chunkserver to Client as a response to GetChunksRequestPayload.
/// Contains chunk which have been requested by Client.
#[derive(Serialize, Deserialize, Debug, ChunkPayload)]
pub struct DownloadChunkResponsePayload {
    pub chunk_id: ChunkId,
    pub chunk_size: u64,
    #[serde(skip)]
    pub offset: u64, // helper for reconstructing the file
    #[serde(skip)]
    pub data: PathBuf,
}

/// Sent from any server to a client when no other response would be sent.
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestStatusPayload {
    Ok,
    InvalidRequest,
    InternalServerError,
}

/// Sent (with/once after logging) from client to MetadataServer
/// (for now, we could offload it to a separate server)
/// to get client's folder structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetClientFolderStructureRequestPayload {}

/// Sent from MetadataServer to Client as a response to GetClientFolderStructureRequestPayload.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetClientFolderStructureResponsePayload {}

/// Sent at the end of the client session (and once every some interval e.g. 10mins)
/// to MetadataServer with any updates to client's folder structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct SendClientFolderStructurePayload {}

impl MessagePayload for AcceptNewChunkServerPayload {}
impl MessagePayload for ChunkServerDiscoverPayload {}
impl MessagePayload for HeartbeatPayload {}
impl MessagePayload for ChunkPlacementRequestPayload {}
impl MessagePayload for GetChunkPlacementRequestPayload {}
impl MessagePayload for ChunkPlacementResponsePayload {}
impl MessagePayload for GetChunkPlacementResponsePayload {}
impl MessagePayload for DownloadChunkRequestPayload {}
impl MessagePayload for RequestStatusPayload {}
impl MessagePayload for GetClientFolderStructureRequestPayload {}
impl MessagePayload for GetClientFolderStructureResponsePayload {}
impl MessagePayload for SendClientFolderStructurePayload {}
