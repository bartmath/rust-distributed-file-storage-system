use serde::{Deserialize, Serialize};
use uuid::Uuid;

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