use crate::commands::CliCommand;
use crate::types::Hostname;
use arc_swap::ArcSwapOption;
use quinn::{Connection, Endpoint};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use storage_core::common::config::MAX_CHUNK_SIZE;
use storage_core::common::{
    ChunkPlacementRequestPayload, ChunkPlacementResponsePayload, ChunkTransfer,
    ChunkserverExternalMessage, DownloadChunkRequestPayload, DownloadChunkResponsePayload,
    GetClientFolderStructureRequestPayload, GetClientFolderStructureResponsePayload,
    GetFilePlacementRequestPayload, GetFilePlacementResponsePayload, Message, MessagePayload,
    MetadataServerExternalMessage, RequestStatusPayload, UpdateClientFolderStructurePayload,
    UploadChunkPayload,
};
use storage_core::dbg_println;
use tokio::fs::File;

pub(super) struct Client {
    metadata_server_addr: SocketAddr,
    metadata_server_hostname: Hostname,

    endpoint: Arc<Endpoint>,

    metadata_server_connection: Arc<ArcSwapOption<Connection>>,
}

impl Client {
    pub(super) fn new(
        metadata_server_addr: SocketAddr,
        metadata_server_hostname: Hostname,
        endpoint: Endpoint,
    ) -> Self {
        Client {
            metadata_server_addr,
            metadata_server_hostname,
            endpoint: Arc::new(endpoint),
            metadata_server_connection: Arc::default(),
        }
    }
    pub(super) async fn handle_command(&self, cmd: CliCommand) -> anyhow::Result<()> {
        match cmd {
            CliCommand::Ls => self.list_all_files().await,
            CliCommand::Upload { path } => self.upload_file(path).await,
            CliCommand::Download {
                file_name,
                destination,
            } => self.download_file(file_name, destination).await,
            CliCommand::Exit => self.end_session().await,
        }
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
                r#"[{}]"#,
                res.all_files
                    .iter()
                    .map(|x| format!("  {}", x))
                    .collect::<Vec<_>>()
                    .join(",\n")
            );
            println!("Files stored in storage\n{}", list_str);
        }

        Ok(())
    }

    async fn upload_file(&self, path: PathBuf) -> anyhow::Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }

        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy()
            .to_string();

        let file_size = tokio::fs::metadata(&path).await?.len();

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

        dbg_println!(
            "Received response for {} chunks",
            res.selected_chunkservers.len()
        );

        // Upload chunks to chunkservers
        let mut offset = 0u64;

        for chunk_location in res.selected_chunkservers {
            let current_chunk_size = std::cmp::min(file_size - offset, MAX_CHUNK_SIZE as u64);

            dbg_println!(
                "Connecting to chunkserver {} {}",
                chunk_location.primary.chunkserver_id,
                chunk_location.primary.server_hostname
            );

            let cs_conn = self
                .endpoint
                .connect(
                    chunk_location.primary.server_location,
                    &*chunk_location.primary.server_hostname,
                )?
                .await?;
            let (mut cs_send, _cs_recv) = cs_conn.open_bi().await?;

            let chunk_transfer = ChunkTransfer::new(Some(offset), path.clone(), false);

            ChunkserverExternalMessage::UploadChunk(UploadChunkPayload {
                chunk_id: chunk_location.chunk_id,
                chunk_size: current_chunk_size,
                chunk_transfer,
            })
            .send(&mut cs_send)
            .await?;

            cs_send.finish()?;

            offset += current_chunk_size;
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
        dbg_println!("Download destination path {}", full_destination.display());
        File::create(full_destination.clone()).await?;

        let mut offset = 0u64;

        for chunk_location in res.chunks_locations {
            let cs_conn = self
                .endpoint
                .connect(
                    chunk_location.primary.server_location,
                    &*chunk_location.primary.server_hostname,
                )?
                .await?;
            let (mut cs_send, mut cs_recv) = cs_conn.open_bi().await?;

            ChunkserverExternalMessage::DownloadChunkRequest(DownloadChunkRequestPayload {
                chunk_id: chunk_location.chunk_id,
            })
            .send(&mut cs_send)
            .await?;

            // Read file's chunks and stream them straight to the file
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

    async fn end_session(&self) -> anyhow::Result<()> {
        let conn = self.get_metadata_connection().await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        let req = MetadataServerExternalMessage::UpdateClientFolderStructure(
            UpdateClientFolderStructurePayload {},
        );
        req.send(&mut send).await?;

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
