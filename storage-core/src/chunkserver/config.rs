use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "server")]
pub(crate) struct ChunkserverOpt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog")]
    pub(crate) keylog: bool,
    /// directory to serve files from
    pub(crate) root: PathBuf,
    /// TLS private key in PEM format
    #[clap(short = 'k', long = "key", requires = "cert")]
    pub(crate) key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    pub(crate) cert: Option<PathBuf>,
    /// Address to listen on for connection from clients.
    #[clap(long = "client-socket-addr", default_value = "[::]:12345")]
    pub(crate) client_socket_addr: SocketAddr,
    /// Address to listen on for connection from internal servers.
    #[clap(long = "internal-socket-addr", default_value = "[::]:12346")]
    pub(crate) internal_socket_addr: SocketAddr,
    /// Metadata server hostname.
    #[clap(long = "metadata-server-hostname")]
    pub(crate) metadata_server_hostname: String,
    /// Metadata server address for internal communication.
    #[clap(long = "metadata-server-addr", default_value = "[::1]:4433")]
    pub(crate) metadata_server_addr: SocketAddr,
    /// Unique identification of the rack the chunkserver is placed in.
    #[clap(long = "rack-id")]
    pub(crate) rack_id: String,
}
