use super::config::ChunkserverOpt;
use crate::external::ChunkserverExternal;
use crate::internal::ChunkserverInternal;
use anyhow::{Result, bail};
use quinn::Endpoint;
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use rustls::pki_types::CertificateDer;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use storage_core::common;

pub(crate) fn chunkserver_setup(
    options: ChunkserverOpt,
) -> Result<(ChunkserverInternal, ChunkserverExternal)> {
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

    let clients_endpoint = Endpoint::server(client_config, options.client_socket_addr)
        .expect("Couldn't create client endpoint");
    let mut internal_endpoint = Endpoint::server(internal_config, options.internal_socket_addr)
        .expect("Couldn't create internal endpoint");

    let path = std::env::current_dir().expect("Couldn't get current directory");
    let cert_path = path.join("../../../cert.der");
    let server_cert_der = std::fs::read(&cert_path).expect("Read");
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(server_cert_der.as_ref()))
        .expect("Couldn't add server certificate");

    let mut client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    client_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    let client_config = quinn::ClientConfig::new(Arc::new(
        QuicClientConfig::try_from(client_crypto).expect("couldn't create client config"),
    ));

    internal_endpoint.set_default_client_config(client_config);

    let internal_endpoint = Arc::new(internal_endpoint);
    let clients_endpoint = Arc::new(clients_endpoint);

    let requests_since_heartbeat = Arc::new(AtomicU64::new(0));
    let chunks = Arc::new(scc::HashMap::new());
    let chunkserver_connections = Arc::new(scc::HashMap::new());

    let internal_chunkserver = ChunkserverInternal::new(
        options.chunkserver_hostname,
        options.rack_id,
        options.advertised_internal_addr,
        options.advertised_external_addr,
        requests_since_heartbeat.clone(),
        chunks.clone(),
        internal_endpoint.clone(),
        options.metadata_server_addr,
        options.metadata_server_hostname,
        chunkserver_connections.clone(),
    );

    let external_chunkserver = ChunkserverExternal::new(
        chunks,
        requests_since_heartbeat,
        clients_endpoint,
        internal_endpoint,
        chunkserver_connections,
    );

    Ok((internal_chunkserver, external_chunkserver))
}
