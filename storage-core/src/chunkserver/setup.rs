use super::config::ChunkserverOpt;
use crate::external::ChunkserverExternal;
use crate::internal::ChunkserverInternal;
use anyhow::Result;
use quinn::Endpoint;
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use rustls::pki_types::CertificateDer;
use rustls_platform_verifier::BuilderVerifierExt;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::fs;
use storage_core::common;
use storage_core::common::config::{FINAL_STORAGE_ROOT, TMP_STORAGE_ROOT};

pub(crate) fn chunkserver_setup(
    options: ChunkserverOpt,
) -> Result<(ChunkserverInternal, ChunkserverExternal)> {
    // Load static variables
    let final_storage_root = std::env::current_dir()?.join(options.final_root);
    let tmp_storage_root = std::env::current_dir()?.join(options.tmp_root);

    fs::create_dir_all(final_storage_root.clone()).expect("Couldn't create final storage root");
    fs::create_dir_all(tmp_storage_root.clone()).expect("Couldn't create tmp storage root");

    FINAL_STORAGE_ROOT
        .set(final_storage_root)
        .expect("Final storage root set failed");
    TMP_STORAGE_ROOT
        .set(tmp_storage_root)
        .expect("Temporary storage root set failed");

    // Set up QUIC endpoints
    let certificate_provider = common::certificate_provider(
        Some(options.chunkserver_hostname.clone()),
        options.key,
        options.cert.clone(),
    )?;
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

    let clients_endpoint = Endpoint::server(client_config, options.client_socket_addr)
        .expect("Couldn't create client endpoint");
    let mut internal_endpoint = Endpoint::server(internal_config, options.internal_socket_addr)
        .expect("Couldn't create internal endpoint");

    let mut client_crypto = rustls::ClientConfig::builder()
        .with_platform_verifier()
        .expect("Could not load platform certificates")
        .with_no_client_auth();

    // TODO: Temporary solution to make servers accept self-signed certificates.
    if options.cert.is_none() {
        let path = std::env::current_dir()
            .expect("Couldn't get current directory")
            .join("certificates")
            .join(options.metadata_server_hostname.clone());
        let cert_path = path.join("cert.der");
        let server_cert_der = std::fs::read(&cert_path).expect("Unable to read certificate");
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(CertificateDer::from(server_cert_der.as_ref()))
            .expect("Couldn't add server certificate");

        client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
    }

    client_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    let client_config = quinn::ClientConfig::new(Arc::new(
        QuicClientConfig::try_from(client_crypto).expect("couldn't create client config"),
    ));

    internal_endpoint.set_default_client_config(client_config);

    // Create servers
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
