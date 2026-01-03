//! Creates and runs **chunkserver**.
//!
//! # Running in Debug Mode
//! - **Behavior:** Prints debugging info about operations and auto-generates self-signed certificates.
//! - **Example usage**
//! ```bash
//!   cargo run --bin chunkserver -- \
//!     --advertised-external-addr [::1]:12345 \
//!     --advertised-internal-addr [::1]:12346 \
//!     --client-socket-addr [::]:12345 \
//!     --internal-socket-addr [::]:12346
//!   ```
//!
//! # Running in Release Mode
//! - **Command:** `cargo run --release --bin chunkserver -- [OPTIONS]`
//! - **Requirements:**
//!   - Valid TLS certificate files (`--cert`, `--key`)
//!   - All required arguments must be provided (see `--help` for full list)
//!   - Run `cargo run --release --bin chunkserver -- --help` for details
//! - **WARNING:** Self-signed certificates are NOT available in release builds for security reasons.

use crate::external::ChunkserverExternal;
use crate::internal::ChunkserverInternal;
use clap::Parser;
use config::ChunkserverOpt;
use setup::chunkserver_setup;
use storage_core::common::QuicServer;

mod chunk;
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
