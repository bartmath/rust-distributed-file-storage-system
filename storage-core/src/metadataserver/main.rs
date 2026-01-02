//! Creates and runs **metadata server**.
//!
//! # Running in Debug Mode
//! - **Behavior:** Prints debugging info about operations and auto-generates self-signed certificates
//! - **Command:** `cargo run --bin metadataserver`
//!
//! # Running in Release Mode
//! - **Command:** `cargo run --release --bin metadataserver -- [OPTIONS]`
//! - **Requirements:**
//!   - Valid TLS certificate files (`--cert`, `--key`)
//!   - All required arguments must be provided (see `--help` for full list)
//!   - Run `cargo run --release --bin metadataserver -- --help` for details
//! - **WARNING:** Self-signed certificates are NOT available in release builds for security reasons.
//!
//! ## Important Note
//! The Metadataserver will **panic** (fail to start or process requests) unless at least
//! **N_CHUNK_REPLICAS + 1** chunkservers are connected.
//!
//! For example, if `N_CHUNK_REPLICAS` is 2, you need **3** connected chunkservers.

use crate::config::MetadataServerOpt;
use crate::external::MetadataServerExternal;
use crate::internal::MetadataServerInternal;
use crate::setup::metadata_server_setup;
use clap::Parser;
use storage_core::common::QuicServer;

mod config;
mod external;
mod internal;
mod setup;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let opt = MetadataServerOpt::parse();
    let (metadata_server_internal, metadata_server_external) =
        metadata_server_setup(opt).expect("Couldn't setup metadata servers");

    run(metadata_server_internal, metadata_server_external).await
}

async fn run(
    internal_chunkserver: MetadataServerInternal,
    external_chunkserver: MetadataServerExternal,
) -> anyhow::Result<()> {
    let internal_handle = tokio::spawn(async move { internal_chunkserver.run().await });
    let external_handle = tokio::spawn(async move { external_chunkserver.run().await });

    // If one of the sides of the server crashes, we want to exit immediately.
    let _ = tokio::try_join!(internal_handle, external_handle)?;
    Ok(())
}
