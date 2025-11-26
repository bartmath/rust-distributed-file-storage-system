use quinn::RecvStream;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_MESSAGE_SIZE: usize = 1024;
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    ChunkServerDiscoverMessage(ChunkServerDiscoverPayload),
    AcceptNewChunkServerMessage(AcceptNewChunkServerPayload),
    HeartbeatMessage(HeartbeatPayload),
    UploadChunkServersRequestMessage(UploadChunkServersRequestPayload),
}

impl Message {
    pub async fn from_stream(
        mut stream: RecvStream,
        buffer_size: usize,
    ) -> anyhow::Result<Message> {
        let buffer = stream.read_to_end(buffer_size).await?;
        Ok(bincode::deserialize::<Message>(&buffer)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkServerDiscoverPayload {
    pub rack_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcceptNewChunkServerPayload {
    pub chunkserver_new_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatPayload {}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkServersRequestPayload {}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunkServersResponsePayload {}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadChunksPayload {}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkserversRequestPayload {}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadChunkserversResponsePayload {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksRequestPayload {}

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
