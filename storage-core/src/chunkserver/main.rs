mod chunkserver;
mod config;
mod setup;

use crate::chunkserver::{ChunkserverExternal, ChunkserverInternal};
use anyhow::Result;
use clap::Parser;
use config::ChunkserverOpt;
use setup::chunkserver_setup;

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
    let opt = ChunkserverOpt::parse();
    let (internal_chunkservers, external_chunkserver) =
        chunkserver_setup(opt).expect("TODO: Couldn't setup chunkservers");

    let code = {
        if let Err(e) = run(internal_chunkservers, external_chunkserver) {
            eprintln!("ERROR: {e}");
            1
        } else {
            0
        }
    };
    std::process::exit(code);
}

#[tokio::main]
async fn run(
    mut internal_chunkserver: ChunkserverInternal,
    mut external_chunkserver: ChunkserverExternal,
) -> Result<()> {
    let internal_handle = tokio::spawn(async move { internal_chunkserver.server_loop().await });
    let external_handle = tokio::spawn(async move { external_chunkserver.server_loop().await });

    // If one of the sides of the server crashes, we want to exit immediately.
    tokio::try_join!(internal_handle, external_handle)?;
    Ok(())
}

/* async fn run_internal_communication_loop(internal_endpoint: Endpoint) -> Result<()> {
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
            tokio::fs::write(payload.chunk_id.to_string(), payload.data).await?;
        }
        DownloadChunkRequest(payload) => {
            todo!("handle download request")
        }
        AcceptNewChunkServer(_) => bail!("Chunkserver was already accepted"),
    };

    Ok(())
}
*/
