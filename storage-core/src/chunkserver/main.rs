mod config;
mod external;
mod internal;
mod setup;

use crate::external::ChunkserverExternal;
use crate::internal::ChunkserverInternal;
use anyhow::Result;
use clap::Parser;
use config::ChunkserverOpt;
use setup::chunkserver_setup;
use storage_core::common::QuicServer;

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
    internal_chunkserver: ChunkserverInternal,
    external_chunkserver: ChunkserverExternal,
) -> Result<()> {
    let internal_handle = tokio::spawn(async move { internal_chunkserver.run().await });
    let external_handle = tokio::spawn(async move { external_chunkserver.run().await });

    // If one of the sides of the server crashes, we want to exit immediately.
    let _ = tokio::try_join!(internal_handle, external_handle)?;
    Ok(())
}
