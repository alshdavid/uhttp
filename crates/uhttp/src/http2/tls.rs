use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;

pub(crate) fn load_certs_and_key(
  cert_path: &Path,
  key_path: &Path,
) -> crate::Result<Arc<ServerConfig>> {
  let cert_file =
    std::fs::File::open(cert_path).map_err(|_| crate::Error::generic("cannot open cert file"))?;

  let mut cert_reader = BufReader::new(cert_file);
  let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
    .collect::<Result<Vec<_>, _>>()
    .map_err(|_| crate::Error::generic("failed to load certs"))?;

  let key_file =
    std::fs::File::open(key_path).map_err(|_| crate::Error::generic("cannot open key file"))?;

  let mut key_reader = BufReader::new(key_file);

  let key =
    std::iter::from_fn(|| rustls_pemfile::read_one(&mut key_reader).transpose()).find_map(|item| {
      match item {
        Ok(rustls_pemfile::Item::Pkcs1Key(k)) => Some(PrivateKeyDer::Pkcs1(k)),
        Ok(rustls_pemfile::Item::Pkcs8Key(k)) => Some(PrivateKeyDer::Pkcs8(k)),
        Ok(rustls_pemfile::Item::Sec1Key(k)) => Some(PrivateKeyDer::Sec1(k)),
        Ok(other) => {
          // Debug: See what is actually being found
          eprintln!("Found non-key PEM item: {:?}", other);
          None
        }
        Err(e) => {
          eprintln!("Error parsing PEM: {}", e);
          None
        }
      }
    });

  let key = match key {
    Some(k) => k,
    None => {
      return Err(crate::Error::generic(
        "No valid private key (PKCS1, PKCS8, or SEC1) found in file",
      ));
    }
  };

  let mut config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .map_err(|_| crate::Error::generic("failed to create TLS config"))?;

  config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

  Ok(Arc::new(config))
}
