use super::types::Hostname;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "server")]
pub(super) struct ChunkserverOpt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog", default_value = "false")]
    pub(super) keylog: bool,
    /// temporary directory to save unverified files to
    #[clap(long = "tmp-root", default_value = "tmp/")]
    pub(super) tmp_root: PathBuf,
    /// final directory to save files to
    #[clap(long = "final-root", default_value = "final/")]
    pub(super) final_root: PathBuf,
    /// TLS private key in PEM format
    #[clap(short = 'k', long = "key", requires = "cert")]
    pub(super) key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    pub(super) cert: Option<PathBuf>,
    /// Chunkserver's hostname for client and other chunkserver to connect to.
    #[clap(long = "chunkserver-hostname", default_value = "chunkserver-1")]
    pub(super) chunkserver_hostname: Hostname,
    /// Address advertised to clients to connect to.
    #[clap(long = "advertised-external-addr", default_value = "[::]:12345")]
    pub(super) advertised_external_addr: SocketAddr,
    /// Address advertised to other chunkservers to connect to.
    #[clap(long = "advertised-internal-addr", default_value = "[::]:12346")]
    pub(super) advertised_internal_addr: SocketAddr,
    /// Address to listen on for connection from clients.
    #[clap(long = "client-socket-addr", default_value = "[::]:12345")]
    pub(super) client_socket_addr: SocketAddr,
    /// Address to listen on for connection from internal servers.
    #[clap(long = "internal-socket-addr", default_value = "[::]:12346")]
    pub(super) internal_socket_addr: SocketAddr,
    /// Metadata server hostname.
    #[clap(long = "metadata-server-hostname", default_value = "metadata-server")]
    pub(super) metadata_server_hostname: Hostname,
    /// Metadata server address for internal communication.
    #[clap(long = "metadata-server-addr", default_value = "[::1]:4433")]
    pub(super) metadata_server_addr: SocketAddr,
    /// Unique identification of the rack the chunkserver is placed in.
    #[clap(long = "rack-id", default_value = "rack-1")]
    pub(super) rack_id: String,
}
