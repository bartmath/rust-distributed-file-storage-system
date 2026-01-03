use crate::common::messages::message_payloads::*;
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use storage_macros::Message;
use tokio::io::AsyncWriteExt;

#[allow(async_fn_in_trait)]
pub trait Message: Serialize + DeserializeOwned {
    async fn send(&self, send: &mut SendStream) -> Result<()>;
    async fn recv(recv: &mut RecvStream) -> Result<Self>;
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum MetadataServerExternalMessage {
    ChunkPlacementRequest(ChunkPlacementRequestPayload),
    GetFilePlacementRequest(GetFilePlacementRequestPayload),
    GetClientFolderStructureRequest(GetClientFolderStructureRequestPayload),
    UpdateClientFolderStructure(UpdateClientFolderStructurePayload),
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum MetadataServerInternalMessage {
    ChunkServerDiscover(ChunkServerDiscoverPayload),
    Heartbeat(HeartbeatPayload),
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum ChunkserverExternalMessage {
    UploadChunk(UploadChunkPayload),
    DownloadChunkRequest(DownloadChunkRequestPayload),
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum ChunkserverInternalMessage {
    AcceptNewChunkserver(AcceptNewChunkServerPayload),
}
