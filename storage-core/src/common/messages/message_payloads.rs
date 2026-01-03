use crate::common::config::TMP_STORAGE_ROOT;
use crate::common::messages::chunk_transfer::ChunkTransfer;
use crate::common::messages::payload::{
    ChunkPayload, SerializablePayload, impl_serializable_payload,
};
use crate::common::types::{ChunkLocations, Hostname};
use async_trait::async_trait;
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::fs::File;
use tokio::io::BufWriter;
use uuid::Uuid;

type ChunkId = Uuid;
type RackId = String;

/// Sent by Chunkserver to MetadataServer as first message (or after reconnection with the MetadataServer).
/// Contains introduction of the chunkserver, with data that make it identifiable
/// and chunks that the Chunkserver stores.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkServerDiscoverPayload {
    pub server_id: Uuid,
    pub hostname: Hostname,
    pub rack_id: RackId,
    pub internal_address: SocketAddr,
    pub external_address: SocketAddr,
    pub stored_chunks: Vec<ChunkId>, // we will probably need to send also some other info about the chunk, not only the id
}
impl_serializable_payload!(ChunkServerDiscoverPayload);

/// Sent from MetadataServer to Chunkserver as a response to ChunkServerDiscoverPayload.
/// Contains new id of the Chunkserver which has been assigned by MetadataServer.
#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptNewChunkServerPayload {
    pub chunkserver_new_id: Uuid,
}
impl_serializable_payload!(AcceptNewChunkServerPayload);

/// Sent regularly by ChunkServer to MetadataServer.
/// Contains all data and statistics required by MetadataServer
/// to make informed decision on chunks distribution between Chunkservers.
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatPayload {
    pub server_id: Uuid,
    /// Number of requests from clients since last heartbeat was sent.
    pub client_requests_count: u64,
    /// Available space on the chunkserver's disk in bytes.
    pub available_space: u64,
}
impl_serializable_payload!(HeartbeatPayload);

/// Sent by Client to MetadataServer.
/// Sends some data about the file to upload so that MetadataServer may decide
/// where to store file's chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkPlacementRequestPayload {
    pub filename: String,
    pub file_size: usize,
}
impl_serializable_payload!(ChunkPlacementRequestPayload);

/// Sent by MetadataServer to Client as a response to UploadChunkServersRequestPayload.
/// Contains list of Chunkservers (with their addresses) where the chunks have to be stored.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkPlacementResponsePayload {
    pub selected_chunkservers: Vec<ChunkLocations>,
}
impl_serializable_payload!(ChunkPlacementResponsePayload);

/// Sent from Client to Chunkserver.
/// Contains a Chunk to be stored on the Chunkserver.
#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkPayload {
    pub chunk_id: ChunkId,
    pub chunk_size: u64,
    #[serde(skip)]
    pub chunk_transfer: ChunkTransfer,
}
#[async_trait]
impl ChunkPayload for UploadChunkPayload {
    type Ctx = ();

    async fn send_chunk(&self, send: &mut SendStream) -> anyhow::Result<()> {
        self.chunk_transfer.send_chunk(self.chunk_size, send).await
    }

    // Performed by chunkserver
    async fn recv_chunk(&mut self, recv: &mut RecvStream, _ctx: &()) -> anyhow::Result<()> {
        let path = TMP_STORAGE_ROOT
            .get()
            .expect("Temporary storage not initialized via config")
            .join(self.chunk_id.to_string());

        // We create the chunk_transfer before creating the file itself, to always drop it in case of any error.
        self.chunk_transfer = ChunkTransfer::new(None, path.clone());

        let file = File::create(&path).await?;
        file.set_len(self.chunk_size).await?;
        let mut writer = BufWriter::with_capacity(self.chunk_size as usize, file);

        self.chunk_transfer
            .recv_chunk(self.chunk_size, &mut writer, recv)
            .await?;

        writer.into_inner().sync_all().await?;

        Ok(())
    }
}

/// Sent from Client to MetadataServer.
/// Contains the file id, which Client wants to download.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetFilePlacementRequestPayload {
    pub filename: String,
}
impl_serializable_payload!(GetFilePlacementRequestPayload);

/// Sent from MetadataServer to Client as a response to DownloadChunkserversRequestPayload.
/// Contains list of Chunkservers (with their addresses) which store file's chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetFilePlacementResponsePayload {
    pub chunks_locations: Vec<ChunkLocations>,
}
impl_serializable_payload!(GetFilePlacementResponsePayload);

/// Sent from Client to ChunkServer.
/// Contains list of chunk ids which it wants to download from the ChunkServer.
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkRequestPayload {
    pub chunk_id: ChunkId,
}
impl_serializable_payload!(DownloadChunkRequestPayload);

/// Sent from Chunkserver to Client as a response to GetChunksRequestPayload.
/// Contains chunk which have been requested by Client.
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkResponsePayload {
    pub chunk_id: ChunkId,
    pub chunk_size: u64,
    #[serde(skip)]
    pub chunk_transfer: ChunkTransfer,
}
#[async_trait]
impl ChunkPayload for DownloadChunkResponsePayload {
    type Ctx = (File, u64);

    async fn send_chunk(&self, send: &mut SendStream) -> anyhow::Result<()> {
        self.chunk_transfer.send_chunk(self.chunk_size, send).await
    }

    async fn recv_chunk(&mut self, recv: &mut RecvStream, _ctx: &Self::Ctx) -> anyhow::Result<()> {
        // We create the chunk_transfer before creating the file itself, to always drop it in case of any error.
        self.chunk_transfer = ChunkTransfer::new(None, path.clone());

        file.set_len(self.chunk_size).await?;
        let mut writer = BufWriter::with_capacity(self.chunk_size as usize, file);

        self.chunk_transfer
            .recv_chunk(self.chunk_size, &mut writer, recv)
            .await?;

        writer.into_inner().sync_all().await?;

        Ok(())
    }
}

/// Sent from any server to a client when no other response would be sent.
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestStatusPayload {
    Ok,
    InvalidRequest,
    InternalServerError,
}
impl_serializable_payload!(RequestStatusPayload);

/// Sent (with/once after logging) from client to MetadataServer
/// (for now, we could offload it to a separate server)
/// to get client's folder structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetClientFolderStructureRequestPayload {}
impl_serializable_payload!(GetClientFolderStructureRequestPayload);

/// Sent from MetadataServer to Client as a response to GetClientFolderStructureRequestPayload.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetClientFolderStructureResponsePayload {}
impl_serializable_payload!(GetClientFolderStructureResponsePayload);

/// Sent at the end of the client session (and once every some interval e.g. 10mins)
/// to MetadataServer with any updates to client's folder structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateClientFolderStructurePayload {}
impl_serializable_payload!(UpdateClientFolderStructurePayload);
