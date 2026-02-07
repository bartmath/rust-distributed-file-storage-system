use crate::common::ALPN_QUIC_HTTP;
use anyhow::Context;
use rustls::pki_types::CertificateDer;
use rustls::{ClientConfig, RootCertStore};
use rustls_platform_verifier::BuilderVerifierExt;
use std::path::PathBuf;

pub fn configure_client_tls(cert_paths: Option<Vec<PathBuf>>) -> anyhow::Result<ClientConfig> {
    let mut client_crypto = if let Some(cert_paths) = cert_paths {
        let mut roots = RootCertStore::empty();
        for path in cert_paths {
            let cert_der = std::fs::read(&path)
                .with_context(|| format!("Unable to read {}", path.display()))?;
            roots
                .add(CertificateDer::from(cert_der.as_ref()))
                .with_context(|| format!("Invalid cert: {}", path.display()))?;
        }

        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth()
    } else {
        eprintln!(
            "WARNING: No custom TLS certificate authority provided (missing --cert flag).\n\
             Falling back to the system's native root certificate store."
        );

        ClientConfig::builder()
            .with_platform_verifier()
            .with_context(|| "Could not load platform certificates")?
            .with_no_client_auth()
    };

    client_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    Ok(client_crypto)
}
