use crate::client::Client;
use crate::config::ClientOpt;
use anyhow::Context;
use quinn::IdleTimeout;
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use rustls_platform_verifier::BuilderVerifierExt;
use std::sync::Arc;
use storage_core::common::ALPN_QUIC_HTTP;
use storage_core::common::config::MAX_CLIENT_IDLE_TIMEOUT;

pub(super) fn setup(options: ClientOpt) -> anyhow::Result<Client> {
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_platform_verifier()
        .expect("Could not load platform certificates")
        .with_no_client_auth();

    #[cfg(debug_assertions)]
    if options.cert.is_none() {
        let mut roots = rustls::RootCertStore::empty();

        // Walk certificates/**/cert.der recursively
        let certs_dir = std::env::current_dir()
            .expect("Couldn't get current directory")
            .join("certificates");

        for entry in walkdir::WalkDir::new(&certs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "cert.der" && e.path().is_file())
        {
            let cert_der = std::fs::read(entry.path())
                .with_context(|| format!("Unable to read {}", entry.path().display()))?;
            roots
                .add(CertificateDer::from(cert_der.as_ref()))
                .with_context(|| format!("Invalid cert: {}", entry.path().display()))?;
        }

        client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
    }

    client_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

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
