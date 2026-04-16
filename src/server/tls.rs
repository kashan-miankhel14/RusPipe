use rcgen::generate_simple_self_signed;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;
use std::sync::Arc;

/// Generate a self-signed TLS cert for localhost and return rustls ServerConfig
pub fn self_signed_tls_config() -> anyhow::Result<Arc<ServerConfig>> {
    let cert = generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])?;

    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();

    let cert_chain = certs(&mut BufReader::new(cert_pem.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut keys = pkcs8_private_keys(&mut BufReader::new(key_pem.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)))?;

    Ok(Arc::new(config))
}
