use crate::config::MetadataServerOpt;
use crate::external::MetadataServerExternal;
use crate::internal::MetadataServerInternal;
use anyhow::Result;
use quinn::Endpoint;
use quinn::crypto::rustls::QuicServerConfig;
use std::sync::Arc;
use storage_core::common;
use storage_core::common::config::{HEARTBEAT_INTERVAL, HEARTBEAT_MARGIN, KEEPALIVE_INTERVAL};

pub(crate) fn metadata_server_setup(
    options: MetadataServerOpt,
) -> Result<(MetadataServerInternal, MetadataServerExternal)> {
    // Set up QUIC endpoints
    let certificate_provider = common::certificate_provider(
        Some(options.hostname.clone()),
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

    let mut internal_transport_config = quinn::TransportConfig::default();
    internal_transport_config
        .max_idle_timeout(Some((HEARTBEAT_INTERVAL + HEARTBEAT_MARGIN).try_into()?))
        .keep_alive_interval(Some(KEEPALIVE_INTERVAL));

    let internal_transport_config = Arc::new(internal_transport_config);

    let mut internal_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(internal_crypto)?));
    internal_config.transport_config(internal_transport_config);

    let client_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(client_crypto)?));

    let internal_endpoint = Endpoint::server(internal_config, options.internal_socket_addr)
        .expect("Couldn't create internal endpoint");
    let clients_endpoint = Endpoint::server(client_config, options.client_socket_addr)
        .expect("Couldn't create client endpoint");

    // Create servers
    let internal_endpoint = Arc::new(internal_endpoint);
    let clients_endpoint = Arc::new(clients_endpoint);

    let active_chunkservers = Arc::new(scc::HashMap::new());
    let chunks = Arc::new(scc::HashMap::new());

    let metadata_server_internal = MetadataServerInternal::new(
        internal_endpoint,
        active_chunkservers.clone(),
        chunks.clone(),
    );

    let metadata_server_external =
        MetadataServerExternal::new(clients_endpoint, active_chunkservers, chunks);

    Ok((metadata_server_internal, metadata_server_external))
}
