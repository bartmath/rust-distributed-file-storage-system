use crate::types::Hostname;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "client")]
pub(super) struct ClientOpt {
    /// TLS certificates in DER format to all servers (both metadataserver and all chunkservers)
    #[clap(short = 'c', long = "cert")]
    pub(super) cert: Vec<PathBuf>,
    /// Address to listen to bind to.
    #[clap(short = 'b', long = "bind-socket-addr", default_value = "[::1]:11111")]
    pub(super) socket_addr: SocketAddr,
    /// Metadata server hostname.
    #[clap(
        short = 'a',
        long = "metadata-server-addr",
        default_value = "[::1]:4422"
    )]
    pub(super) metadata_server_addr: SocketAddr,
    /// Metadata server hostname.
    #[clap(long = "hostname", default_value = "metadata-server")]
    pub(super) metadata_server_hostname: Hostname,
}
