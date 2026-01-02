use crate::client::Client;
use crate::config::ClientOpt;
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use rustls_platform_verifier::BuilderVerifierExt;
use std::sync::Arc;
use storage_core::common::ALPN_QUIC_HTTP;

pub(super) fn setup(options: ClientOpt) -> anyhow::Result<Client> {
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_platform_verifier()
        .expect("Could not load platform certificates")
        .with_no_client_auth();

    #[cfg(debug_assertions)]
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

    client_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    let client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));
    let mut endpoint = quinn::Endpoint::client(options.socket_addr)?;
    endpoint.set_default_client_config(client_config);

    let client = Client::new();

    Ok(client)
}
