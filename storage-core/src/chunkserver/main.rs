mod chunkserver;

use anyhow::{Error, Result, bail};
use chunkserver::ChunkServer;
use clap::Parser;
use quinn::crypto::rustls::QuicServerConfig;
use quinn::{Connecting, Endpoint};
use std::time::Duration;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use storage_core::common;
use storage_core::common::ChunkserverMessage::{
    AcceptNewChunkServer, DownloadChunkRequest, UploadChunk,
};
use storage_core::common::{ChunkserverMessage, Message};
use tokio::fs;
use tracing::info;
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
    /// Metadata server address for internal communication.
    #[clap(long = "metadata server address", default_value = "[::1]:4433")]
    metadata_server_addr: SocketAddr,
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
    let chunkserver = setup(opt).expect("TODO: panic message");

    let code = {
        if let Err(e) = run(chunkserver) {
            eprintln!("ERROR: {e}");
            1
        } else {
            0
        }
    };
    std::process::exit(code);
}

fn setup(options: Opt) -> Result<ChunkServer> {
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

    Ok(ChunkServer::new(
        Uuid::new_v4(),
        client_config,
        options.client_socket_addr,
        Duration::from_secs(60),
        internal_config,
        options.internal_socket_addr,
        options.metadata_server_addr,
    ))
}

#[tokio::main]
async fn run(mut chunkserver: ChunkServer) -> Result<()> {
    chunkserver.establish_metadata_server_connection().await?;

    let internal_clone = chunkserver.internal_endpoint.clone();
    tokio::spawn(async move { run_internal_communication_loop(internal_clone) });
    tokio::spawn(async move {
        run_client_communication_loop(chunkserver.client_endpoint, chunkserver.internal_endpoint)
    });
    Ok(())
}

async fn run_internal_communication_loop(internal_endpoint: Endpoint) -> Result<()> {
    Ok(())
}

async fn run_client_communication_loop(
    client_endpoint: Endpoint,
    internal_endpoint: Endpoint,
) -> Result<()> {
    loop {
        if let Some(incoming) = client_endpoint.accept().await {
            if let Ok(conn) = incoming.accept() {
                let cln = internal_endpoint.clone();
                tokio::spawn(async move { handle_conn(conn, cln) });
            }
        }
    }
}

async fn handle_conn(client_connecting: Connecting, internal_endpoint: Endpoint) -> Result<()> {
    let connection = client_connecting.await?;
    loop {
        let stream = connection.accept_bi().await;
        let stream = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                info!("connection closed");
                return Ok(());
            }
            Err(e) => {
                return Err(Error::from(e));
            }
            Ok(s) => s,
        };
        tokio::spawn(async move { handle_request(stream).await });
    }
}

async fn handle_request(
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    match ChunkserverMessage::recv(&mut recv).await? {
        UploadChunk(payload) => {
            // TODO: to make everything faster, we may consider omitting the save to Vec,
            // TODO: and stream the data to the file straight away
            fs::write(payload.chunk_id.to_string(), payload.data).await?;
        }
        DownloadChunkRequest(payload) => {
            todo!("handle download request")
        }
        AcceptNewChunkServer(_) => bail!("Chunkserver was already accepted"),
    };

    Ok(())
}
