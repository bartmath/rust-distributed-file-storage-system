use crate::client::Client;
use crate::config::ClientOpt;
use quinn::IdleTimeout;
use quinn::crypto::rustls::QuicClientConfig;
use std::sync::Arc;
use storage_core::common::config::MAX_CLIENT_IDLE_TIMEOUT;
use storage_core::common::configure_client_tls;

pub(super) fn setup(options: ClientOpt) -> anyhow::Result<Client> {
    let cert_paths = (!options.cert.is_empty()).then_some(options.cert);
    #[cfg(debug_assertions)]
    let cert_paths = cert_paths.or_else(|| {
        // Walk certificates/**/cert.der recursively
        let certs_dir = std::env::current_dir()
            .expect("Couldn't get current directory")
            .join("certificates");

        let found_cert_paths = walkdir::WalkDir::new(&certs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "cert.der" && e.path().is_file())
            .map(|e| e.path().to_owned())
            .collect::<Vec<_>>();

        (!found_cert_paths.is_empty()).then_some(found_cert_paths)
    });

    let client_crypto = configure_client_tls(cert_paths)?;

    let mut transport = quinn::TransportConfig::default();
    transport.max_idle_timeout(Some(IdleTimeout::try_from(MAX_CLIENT_IDLE_TIMEOUT)?));
    let mut client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));
    client_config.transport_config(Arc::new(transport));

    let mut endpoint = quinn::Endpoint::client(options.socket_addr)?;
    endpoint.set_default_client_config(client_config);

    let client = Client::new(
        options.metadata_server_addr,
        options.metadata_server_hostname,
        endpoint,
    );

    Ok(client)
}
