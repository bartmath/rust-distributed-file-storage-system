use anyhow::{Context, Result, bail};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::path::PathBuf;
use std::{fs, io};
use tracing::info;

/// Trait for providing TLS certificates to a server.
pub trait CertificateProvider {
    fn get_certificate(&self) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>;
}

pub fn certificate_provider(
    key: Option<PathBuf>,
    cert: Option<PathBuf>,
) -> Result<Box<dyn CertificateProvider>> {
    if let (Some(key_path), Some(cert_path)) = (key, cert) {
        Ok(Box::new(FileCertificateProvider::new(cert_path, key_path)))
    } else {
        #[cfg(debug_assertions)]
        {
            Ok(Box::new(SelfSignedCertificateProvider {}))
        }
        #[cfg(not(debug_assertions))]
        {
            bail!("No TLS certificate files provided in release mode");
        }
    }
}

struct FileCertificateProvider {
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl FileCertificateProvider {
    fn new(cert_path: PathBuf, key_path: PathBuf) -> FileCertificateProvider {
        FileCertificateProvider {
            cert_path,
            key_path,
        }
    }
}

impl CertificateProvider for FileCertificateProvider {
    fn get_certificate(&self) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        let key = if self.key_path.extension().is_some_and(|x| x == "der") {
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                fs::read(&self.key_path).context("failed to read private key file")?,
            ))
        } else {
            PrivateKeyDer::from_pem_file(&self.key_path)
                .context("failed to read PEM from private key file")?
        };

        let cert_chain = if self.cert_path.extension().is_some_and(|x| x == "der") {
            vec![CertificateDer::from(
                fs::read(&self.cert_path).context("failed to read certificate chain file")?,
            )]
        } else {
            CertificateDer::pem_file_iter(&self.cert_path)
                .context("failed to read PEM from certificate chain file")?
                .collect::<Result<_, _>>()
                .context("invalid PEM-encoded certificate")?
        };

        Ok((cert_chain, key))
    }
}

#[cfg(debug_assertions)]
struct SelfSignedCertificateProvider;

#[cfg(debug_assertions)]
impl CertificateProvider for SelfSignedCertificateProvider {
    fn get_certificate(&self) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        let path = std::env::current_dir()?;
        let cert_path = path.join("../../../cert.der");
        let key_path = path.join("../../../key.der");
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Ok((cert, key)) => (
                CertificateDer::from(cert),
                PrivateKeyDer::try_from(key).map_err(anyhow::Error::msg)?,
            ),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("generating self-signed certificate");
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
                let key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
                let cert = cert.cert.into();
                fs::create_dir_all(path).context("failed to create certificate directory")?;
                fs::write(&cert_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, key.secret_pkcs8_der())
                    .context("failed to write private key")?;
                (cert, key.into())
            }
            Err(e) => {
                bail!("failed to read certificate: {}", e);
            }
        };

        println!("files created");
        Ok((vec![cert], key))
    }
}
