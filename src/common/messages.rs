use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkServerDiscoverMessage {
    pub rack_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptNewChunkServerMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatMessage {
    pub rack_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkServersRequestMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkServersResponseMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunksMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkserversRequestMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkserversResponseMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksRequestMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksResponseMessage {
}

// TODO: SECOND PHASE

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderRequestMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteFileRequestMessage {
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesInFolderRequestMessage {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesInFolderResponseMessage {
}

