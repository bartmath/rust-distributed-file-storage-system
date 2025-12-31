mod chunk;
mod config;
mod external;
mod internal;
mod setup;
mod types;

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

    let opt = ChunkserverOpt::parse();
    let (internal_chunkservers, external_chunkserver) =
        chunkserver_setup(opt).expect("TODO: Couldn't setup chunkservers");

    let _ = run(internal_chunkservers, external_chunkserver).expect("Server crashed");
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
