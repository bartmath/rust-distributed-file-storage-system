pub mod config;
mod dbg_println;
pub mod message;
mod server;
pub mod types;

pub use message::MessagePayload;
pub use message::chunk_transfer::ChunkTransfer;
pub use message::message_payloads::*;
pub use message::messages::*;
pub use server::{CertificateProvider, QuicServer, certificate_provider, configure_client_tls};

#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];
