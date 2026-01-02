use super::types::Hostname;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "server")]
pub(super) struct MetadataServerOpt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog", default_value = "false")]
    pub(super) keylog: bool,
    /// TLS private key in PEM format
    #[clap(short = 'k', long = "key", requires = "cert")]
    pub(super) key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    pub(super) cert: Option<PathBuf>,
    /// Address to listen on for connection from clients.
    #[clap(long = "client-socket-addr", default_value = "[::1]:4422")]
    pub(super) client_socket_addr: SocketAddr,
    /// Address to listen on for connection from internal servers.
    #[clap(long = "internal-socket-addr", default_value = "[::1]:4433")]
    pub(super) internal_socket_addr: SocketAddr,
    /// Metadata server hostname.
    #[clap(long = "hostname", default_value = "metadata-server")]
    pub(super) hostname: Hostname,
}
