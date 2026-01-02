mod chunk;
mod config;
mod external;
mod internal;
mod setup;
mod types;

use crate::external::ChunkserverExternal;
use crate::internal::ChunkserverInternal;
use clap::Parser;
use config::ChunkserverOpt;
use setup::chunkserver_setup;
use storage_core::common::QuicServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let opt = ChunkserverOpt::parse();
    let (internal_chunkservers, external_chunkserver) =
        chunkserver_setup(opt).expect("Couldn't setup chunkservers");

    run(internal_chunkservers, external_chunkserver).await
}

async fn run(
    internal_chunkserver: ChunkserverInternal,
    external_chunkserver: ChunkserverExternal,
) -> anyhow::Result<()> {
    let internal_handle = tokio::spawn(async move { internal_chunkserver.run().await });
    let external_handle = tokio::spawn(async move { external_chunkserver.run().await });

    // If one of the sides of the server crashes, we want to exit immediately.
    let _ = tokio::try_join!(internal_handle, external_handle)?;
    Ok(())
}
