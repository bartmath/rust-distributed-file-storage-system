mod chunkserver;

use anyhow::{Result, bail};
use chunkserver::ChunkServer;
use clap::Parser;
use quinn::crypto::rustls::QuicServerConfig;
use rust_dfss::common;
use std::time::Duration;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[clap(name = "server")]
struct Opt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog")]
    keylog: bool,
    /// directory to serve files from
    root: PathBuf,
    /// TLS private key in PEM format
    #[clap(short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    cert: Option<PathBuf>,
    /// Enable stateless retries
    #[clap(long = "stateless-retry")]
    stateless_retry: bool,
    /// Address to listen on for connection from clients.
    #[clap(long = "client socket address", default_value = "[::]:12345")]
    client_socket_addr: SocketAddr,
    /// Address to listen on for connection from internal servers.
    #[clap(long = "internal socket address", default_value = "[::]:12346")]
    internal_socket_addr: SocketAddr,
    /// Maximum number of concurrent connections to allow
    #[clap(long = "connection-limit")]
    connection_limit: Option<usize>,
}

fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();
    let opt = Opt::parse();
    let code = {
        if let Err(e) = run(opt) {
            eprintln!("ERROR: {e}");
            1
        } else {
            0
        }
    };
    std::process::exit(code);
}

#[tokio::main]
async fn run(options: Opt) -> Result<()> {
    let certificate_provider = common::certificate_provider(options.key, options.cert)?;
    let (certs, key) = certificate_provider.get_certificate()?;

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    server_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    if options.keylog {
        server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());
    }

    let client_crypto = server_crypto.clone();
    let internal_crypto = server_crypto;

    let mut client_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(client_crypto)?));
    let transport_config = Arc::get_mut(&mut client_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let internal_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(internal_crypto)?));

    let root = Arc::<Path>::from(options.root.clone());
    if !root.exists() {
        bail!("root path does not exist");
    }

    eprintln!("listening on");

    let mut chunkserver = ChunkServer::new(
        Uuid::new_v4(),
        client_config,
        options.client_socket_addr,
        Duration::from_secs(60),
        internal_config,
        options.internal_socket_addr,
    );

    chunkserver.establish_metadata_server_connection().await?;
    Ok(())
}
