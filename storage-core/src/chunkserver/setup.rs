use super::config::ChunkserverOpt;
use crate::external::ChunkserverExternal;
use crate::internal::ChunkserverInternal;
use anyhow::Result;
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::{Endpoint, IdleTimeout};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use storage_core::common;
use storage_core::common::config::{
    FINAL_STORAGE_ROOT, HEARTBEAT_INTERVAL, HEARTBEAT_MARGIN, MAX_CLIENT_IDLE_TIMEOUT,
    TMP_STORAGE_ROOT,
};
use storage_core::common::configure_client_tls;

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

    let external_crypto = server_crypto.clone();
    let internal_crypto = server_crypto;

    let mut external_transport_config = quinn::TransportConfig::default();
    external_transport_config
        .max_concurrent_uni_streams(0_u8.into())
        .max_idle_timeout(Some(IdleTimeout::try_from(MAX_CLIENT_IDLE_TIMEOUT)?));
    let mut external_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(external_crypto)?));
    external_config.transport_config(Arc::new(external_transport_config));

    let mut internal_transport_config = quinn::TransportConfig::default();
    internal_transport_config.max_idle_timeout(Some(IdleTimeout::try_from(
        HEARTBEAT_INTERVAL + HEARTBEAT_MARGIN,
    )?));
    let mut internal_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(internal_crypto)?));
    internal_config.transport_config(Arc::new(internal_transport_config));

    let external_endpoint = Endpoint::server(external_config, options.client_socket_addr)
        .expect("Couldn't create client endpoint");
    let mut internal_endpoint = Endpoint::server(internal_config, options.internal_socket_addr)
        .expect("Couldn't create internal endpoint");

    let metadata_server_cert_path = options.metadata_server_certificate;
    #[cfg(debug_assertions)]
    let metadata_server_cert_path = metadata_server_cert_path.or_else(|| {
        Some(
            std::env::current_dir()
                .expect("Couldn't get current directory")
                .join("certificates")
                .join(&options.metadata_server_hostname)
                .join("cert.der"),
        )
    });

    let client_crypto = configure_client_tls(metadata_server_cert_path.map(|p| vec![p]))?;

    let mut client_transport_config = quinn::TransportConfig::default();
    client_transport_config.max_idle_timeout(Some(IdleTimeout::try_from(MAX_CLIENT_IDLE_TIMEOUT)?));
    let mut client_config = quinn::ClientConfig::new(Arc::new(
        QuicClientConfig::try_from(client_crypto).expect("couldn't create client config"),
    ));
    client_config.transport_config(Arc::new(client_transport_config));

    internal_endpoint.set_default_client_config(client_config);

    // Create servers
    let internal_endpoint = Arc::new(internal_endpoint);
    let external_endpoint = Arc::new(external_endpoint);

    let requests_since_heartbeat = Arc::new(AtomicU64::new(0));
    let chunks = Arc::new(scc::HashMap::new());
    let chunkserver_connections = Arc::new(scc::HashMap::new());

    let internal_chunkserver = ChunkserverInternal::new(
        (options.chunkserver_hostname, options.rack_id),
        (
            options.advertised_internal_addr,
            options.advertised_external_addr,
        ),
        requests_since_heartbeat.clone(),
        chunks.clone(),
        internal_endpoint.clone(),
        (
            options.metadata_server_addr,
            options.metadata_server_hostname,
        ),
        chunkserver_connections.clone(),
    );

    let external_chunkserver = ChunkserverExternal::new(
        chunks,
        requests_since_heartbeat,
        external_endpoint,
        internal_endpoint,
        chunkserver_connections,
    );

    Ok((internal_chunkserver, external_chunkserver))
}
