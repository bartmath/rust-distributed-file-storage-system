use crate::commands::CliCommand;
use crate::types::{ChunkId, ChunkserverId, Hostname};
use arc_swap::ArcSwapOption;
use futures::{StreamExt, stream};
use moka::future::Cache;
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use storage_core::common::config::{
    MAX_CHUNK_SIZE, MAX_CLIENT_IDLE_TIMEOUT, MAX_SPAWNED_TASKS, N_CHUNK_REPLICAS,
};
use storage_core::common::types::{ChunkLocations, ChunkserverLocation};
use storage_core::common::{
    ChunkPlacementRequestPayload, ChunkPlacementResponsePayload, ChunkTransfer,
    ChunkserverExternalMessage, DownloadChunkRequestPayload, DownloadChunkResponsePayload,
    GetClientFolderStructureRequestPayload, GetClientFolderStructureResponsePayload,
    GetFilePlacementRequestPayload, GetFilePlacementResponsePayload, Message, MessagePayload,
    MetadataServerExternalMessage, RequestStatusPayload, UpdateClientFolderStructurePayload,
    UploadChunkPayload,
};
use tokio::fs::File;

pub(super) struct Client {
    metadata_server_addr: SocketAddr,
    metadata_server_hostname: Arc<Hostname>,

    endpoint: Arc<Endpoint>,

    metadata_server_connection: Arc<ArcSwapOption<Connection>>,

    chunkserver_connections: Cache<ChunkserverId, Arc<Connection>>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            metadata_server_addr: self.metadata_server_addr,
            metadata_server_hostname: self.metadata_server_hostname.clone(),
            endpoint: self.endpoint.clone(),
            metadata_server_connection: self.metadata_server_connection.clone(),
            chunkserver_connections: self.chunkserver_connections.clone(),
        }
    }
}

impl Client {
    pub(super) fn new(
        metadata_server_addr: SocketAddr,
        metadata_server_hostname: Hostname,
        endpoint: Endpoint,
    ) -> Self {
        Client {
            metadata_server_addr,
            metadata_server_hostname: Arc::new(metadata_server_hostname),
            endpoint: Arc::new(endpoint),
            metadata_server_connection: Arc::default(),
            chunkserver_connections: Cache::builder()
                .max_capacity(256)
                .time_to_live(MAX_CLIENT_IDLE_TIMEOUT)
                .build(),
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
            CliCommand::Exit => self.close_session().await,
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

    async fn get_chunkserver_stream(
        &self,
        chunkserver_location: ChunkserverLocation,
    ) -> anyhow::Result<(SendStream, RecvStream)> {
        // Opens streams on either existing connection or on newly created connection
        // (lambda is run if the connection wasn't in Cache).
        self.chunkserver_connections
            .try_get_with::<_, anyhow::Error>(chunkserver_location.id, async {
                let conn = Arc::new(
                    self.endpoint
                        .connect(chunkserver_location.addr, &*chunkserver_location.hostname)?
                        .await?,
                );
                self.chunkserver_connections
                    .insert(chunkserver_location.id, conn.clone())
                    .await;
                Ok(conn)
            })
            .await
            .map_err(|_| anyhow::anyhow!("Cache error"))?
            .open_bi()
            .await
            .map_err(|_| anyhow::anyhow!("Cache error"))
    }

    async fn upload_chunk(
        self,
        offset: u64,
        file_path: PathBuf,
        chunk_id: ChunkId,
        chunk_size: u64,
        chunkserver: ChunkserverLocation,
    ) -> anyhow::Result<()> {
        let (mut send, _) = self.get_chunkserver_stream(chunkserver).await?;

        let chunk_transfer = ChunkTransfer::new(Some(offset), file_path, false);

        ChunkserverExternalMessage::UploadChunk(UploadChunkPayload {
            chunk_id,
            chunk_size,
            chunk_transfer,
        })
        .send(&mut send)
        .await?;

        send.finish()?;
        Ok(())
    }

    async fn batch_upload_chunks<F>(
        &self,
        file_path: PathBuf,
        file_size: u64,
        chunk_locations: Vec<ChunkLocations>,
        mut chunkserver_extractor: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(ChunkLocations) -> Option<(ChunkId, ChunkserverLocation)>,
    {
        let mut upload_tasks = stream::iter(chunk_locations.into_iter().enumerate().filter_map(
            |(id, chunk_location)| {
                let client = self.clone();
                let file_path = file_path.clone();
                let offset = (MAX_CHUNK_SIZE * id) as u64;
                let chunk_size = std::cmp::min(file_size - offset, MAX_CHUNK_SIZE as u64);
                let (chunk_id, chunkserver) = chunkserver_extractor(chunk_location)?;

                Some(tokio::spawn(async move {
                    client
                        .upload_chunk(offset, file_path, chunk_id, chunk_size, chunkserver)
                        .await
                }))
            },
        ))
        .buffer_unordered(MAX_SPAWNED_TASKS);

        // Process results as they finish
        while let Some(result) = upload_tasks.next().await {
            result??;
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
        self.batch_upload_chunks(
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
            self.batch_upload_chunks(
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
            let (mut cs_send, mut cs_recv) = self
                .get_chunkserver_stream(chunk_location.primary.clone())
                .await?;

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

    pub(super) async fn close_session(&self) -> anyhow::Result<()> {
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
