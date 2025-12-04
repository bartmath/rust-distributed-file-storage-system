use crate::common::message_payload::*;
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use storage_macros::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(async_fn_in_trait)]
pub trait Message: Serialize + DeserializeOwned {
    async fn send(&self, send: &mut SendStream) -> Result<()>;
    async fn recv(recv: &mut RecvStream) -> Result<Self>;
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum MetadataServerMessage {
    ChunkServerDiscover(ChunkServerDiscoverPayload),
    Heartbeat(HeartbeatPayload),
    ChunkPlacementRequest(ChunkPlacementRequestPayload),
    GetChunkPlacementRequest(GetChunkPlacementRequestPayload),
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum ChunkserverMessage {
    AcceptNewChunkServer(AcceptNewChunkServerPayload),
    UploadChunk(UploadChunkPayload),
    DownloadChunkRequest(DownloadChunkRequestPayload),
}

#[derive(Debug, Serialize, Deserialize, Message)]
pub enum ClientMessage {
    ChunkPlacementResponse(ChunkPlacementResponsePayload),
    GetChunkPlacementResponse(GetChunkPlacementResponsePayload),
    DownloadChunkResponse(DownloadChunkResponsePayload),
}
