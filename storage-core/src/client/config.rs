use crate::types::Hostname;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "client")]
pub(super) struct ClientOpt {
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    pub(super) cert: Option<PathBuf>,
    /// Address to listen to bind to.
    #[clap(
        short = 'a',
        long = "client-socket-addr",
        default_value = "[::1]:11111"
    )]
    pub(super) socket_addr: SocketAddr,
    /// Metadata server hostname.
    #[clap(long = "hostname", default_value = "metadata-server")]
    pub(super) metadata_server_hostname: Hostname,
}
