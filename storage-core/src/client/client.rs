use crate::chunkserver_connection_pool::ChunkserverConnectionPool;
use crate::client_chunk_uploader::ClientChunkUploader;
use crate::commands::CliCommand;
use crate::types::Hostname;
use arc_swap::ArcSwapOption;
use quinn::{Connection, Endpoint};
use rand::seq::IndexedRandom;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use storage_core::common::config::{MAX_CLIENT_IDLE_TIMEOUT, N_CHUNK_REPLICAS};
use storage_core::common::{
    ChunkPlacementRequestPayload, ChunkPlacementResponsePayload, ChunkserverExternalMessage,
    DownloadChunkRequestPayload, DownloadChunkResponsePayload,
    GetClientFolderStructureRequestPayload, GetClientFolderStructureResponsePayload,
    GetFilePlacementRequestPayload, GetFilePlacementResponsePayload, Message, MessagePayload,
    MetadataServerExternalMessage, RequestStatusPayload, UpdateClientFolderStructurePayload,
};
use tokio::fs::File;

pub(super) enum LoopAction {
    Continue,
    Exit,
}

pub(super) struct Client {
    metadata_server_addr: SocketAddr,
    metadata_server_hostname: Arc<Hostname>,

    endpoint: Arc<Endpoint>,

    metadata_server_connection: Arc<ArcSwapOption<Connection>>,

    chunkserver_connection_pool: ChunkserverConnectionPool,
    client_chunks_uploader: ClientChunkUploader,
}

impl Client {
    pub(super) fn new(
        metadata_server_addr: SocketAddr,
        metadata_server_hostname: Hostname,
        endpoint: Endpoint,
    ) -> Self {
        let endpoint = Arc::new(endpoint);
        let pool = ChunkserverConnectionPool::new(endpoint.clone(), 256, MAX_CLIENT_IDLE_TIMEOUT);

        Client {
            metadata_server_addr,
            metadata_server_hostname: Arc::new(metadata_server_hostname),
            endpoint,
            metadata_server_connection: Arc::default(),
            chunkserver_connection_pool: pool.clone(),
            client_chunks_uploader: ClientChunkUploader::new(pool),
        }
    }
    pub(super) async fn handle_command(&self, cmd: CliCommand) -> anyhow::Result<LoopAction> {
        match cmd {
            CliCommand::Ls => self.list_all_files().await,
            CliCommand::Upload { path } => self.upload_file(path).await,
            CliCommand::Download {
                file_name,
                destination,
            } => self.download_file(file_name, destination).await,
            CliCommand::Exit => {
                return Ok(LoopAction::Exit);
            }
        }?;

        Ok(LoopAction::Continue)
    }

    async fn get_metadata_connection(&self) -> anyhow::Result<Arc<Connection>> {
        if let Some(conn) = self.metadata_server_connection.load_full() {
            return Ok(conn);
        }

        let conn = self
            .endpoint
            .connect(self.metadata_server_addr, &self.metadata_server_hostname)?
            .await?;

        let conn = Arc::new(conn);
        self.metadata_server_connection.store(Some(conn.clone()));
        Ok(conn)
    }

    // TODO: In future, we will fetch user folder structure in the beginning of their connection
    // TODO: and add option to move around it, sending any updates to the folder structure every 5 minutes.
    async fn list_all_files(&self) -> anyhow::Result<()> {
        let conn = self.get_metadata_connection().await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        MetadataServerExternalMessage::GetClientFolderStructureRequest(
            GetClientFolderStructureRequestPayload {},
        )
        .send(&mut send)
        .await?;

        let res = GetClientFolderStructureResponsePayload::recv_payload(&mut recv, &()).await?;

        if res.all_files.is_empty() {
            println!("You haven't saved any files yet.");
        } else {
            let list_str = format!(
                "[\n{}\n]",
                res.all_files
                    .iter()
                    .map(|x| format!("\t{},", x))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            println!("Files stored in storage:\n{}", list_str);
        }

        Ok(())
    }

    async fn upload_file(&self, file_path: PathBuf) -> anyhow::Result<()> {
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", file_path));
        }

        let file_name = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy()
            .to_string();

        let file_size = tokio::fs::metadata(&file_path).await?.len();

        let conn = self.get_metadata_connection().await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        // Request metadata server for locations the file's chunks should be placed on
        MetadataServerExternalMessage::ChunkPlacementRequest(ChunkPlacementRequestPayload {
            filename: file_name,
            file_size: file_size as usize,
        })
        .send(&mut send)
        .await?;

        let res = ChunkPlacementResponsePayload::recv_payload(&mut recv, &()).await?;
        let chunk_locations = res.selected_chunkservers;

        // Upload chunks to chunkservers
        self.client_chunks_uploader
            .batch_upload_chunks(
                file_path.clone(),
                file_size,
                chunk_locations.clone(),
                move |chunk_locs| Some((chunk_locs.chunk_id, chunk_locs.primary)),
            )
            .await?;

        // We upload the file's chunks to all secondary locations.
        // The closure now returns Option, because the number of secondary chunkservers
        // may be smaller due to some internal metadata server error.
        // In the future, it will be chunkserver's task to replicate from the primary
        // and the client will only upload to primary chunkserver.
        for i in 0..N_CHUNK_REPLICAS {
            self.client_chunks_uploader
                .batch_upload_chunks(
                    file_path.clone(),
                    file_size,
                    chunk_locations.clone(),
                    move |chunk_locs| {
                        chunk_locs
                            .replicas
                            .get(i)
                            .map(|replica| (chunk_locs.chunk_id, replica.clone()))
                    },
                )
                .await?;
        }

        println!("Upload complete.");
        Ok(())
    }

    async fn download_file(&self, file_name: String, destination: PathBuf) -> anyhow::Result<()> {
        let conn = self.get_metadata_connection().await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        // Find chunkservers which store file's chunks.
        MetadataServerExternalMessage::GetFilePlacementRequest(GetFilePlacementRequestPayload {
            filename: file_name.clone(),
        })
        .send(&mut send)
        .await?;

        let res = GetFilePlacementResponsePayload::recv_payload(&mut recv, &()).await?;

        // Prepare destination file (create empty)
        // It will be opened in Write mode and seeked.
        let full_destination = destination.join(file_name);
        File::create(full_destination.clone()).await?;

        let mut offset = 0u64;

        for chunk_location in res.chunks_locations {
            // Load Balancing: Randomly select a replica to distribute read traffic
            // instead of always hammering one chunkserver.
            // We fallback to the primary if the replicas list is empty.
            let target_server = chunk_location
                .replicas
                .choose(&mut rand::rng())
                .unwrap_or(&chunk_location.primary)
                .clone();

            let (mut cs_send, mut cs_recv) = self
                .chunkserver_connection_pool
                .get_chunkserver_stream(target_server)
                .await?;

            ChunkserverExternalMessage::DownloadChunkRequest(DownloadChunkRequestPayload {
                chunk_id: chunk_location.chunk_id,
            })
            .send(&mut cs_send)
            .await?;

            // Receive file's chunks and stream them straight to the file
            let chunk_resp = DownloadChunkResponsePayload::recv_payload(
                &mut cs_recv,
                &(full_destination.clone(), offset), // Context for ChunkPayload::recv_chunk
            )
            .await?;

            offset += chunk_resp.chunk_size;
        }

        println!("Download complete.");
        Ok(())
    }

    pub(super) async fn close_session(&self) -> anyhow::Result<()> {
        let conn = self.get_metadata_connection().await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        MetadataServerExternalMessage::UpdateClientFolderStructure(
            UpdateClientFolderStructurePayload {},
        )
        .send(&mut send)
        .await?;

        let status = RequestStatusPayload::recv_payload(&mut recv, &()).await?;

        match status {
            RequestStatusPayload::Ok => Ok(()),
            _ => Err(anyhow::anyhow!(
                "Session closing failed with status: {:?}",
                status
            )),
        }
    }
}
