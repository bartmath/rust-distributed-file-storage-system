use crate::commands::CliCommand;
use crate::types::Hostname;
use quinn::Endpoint;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Client {
    // metadata_server_addr: SocketAddr,
    // metadata_server_hostname: Hostname,
    endpoint: Arc<Endpoint>,
    // metadata_server_connection: Arc<ArcSwap<Option<Connection>>>,
}

impl Client {
    pub(super) fn new(endpoint: Arc<Endpoint>) -> Self {
        Client { endpoint }
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

    // TODO: In future, we will fetch user folder structure in the beginning of their connection
    // TODO: and add option to move around it, sending any updates to the folder structure every 5 minutes.
    async fn list_all_files(&self) -> anyhow::Result<()> {
        todo!("implement list_all_files");
    }

    async fn upload_file(&self, path: PathBuf) -> anyhow::Result<()> {
        todo!("implement upload_file");
    }

    async fn download_file(&self, file_name: String, destination: PathBuf) -> anyhow::Result<()> {
        todo!("implement download_file");
    }

    async fn end_session(&self) -> anyhow::Result<()> {
        todo!("implement end_session");
    }
}
