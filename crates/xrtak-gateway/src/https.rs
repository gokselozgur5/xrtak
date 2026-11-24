use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio_rustls::TlsAcceptor;

use crate::GatewayState;

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

pub fn generate_self_signed_server_config(extra_sans: &[String]) -> Result<ServerConfig, DynError> {
    let key_pair = rcgen::KeyPair::generate()?;
    let mut subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    for san in extra_sans {
        let trimmed = san.trim();
        if !trimmed.is_empty()
            && !subject_alt_names
                .iter()
                .any(|s| s.eq_ignore_ascii_case(trimmed))
        {
            subject_alt_names.push(trimmed.to_string());
        }
    }

    let mut params = rcgen::CertificateParams::new(subject_alt_names)?;
    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        "XR-TAK Self-Signed WebXR Gateway",
    );
    let cert = params.self_signed(&key_pair)?;

    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der = PrivateKeyDer::Pkcs8(rustls_pki_types::PrivatePkcs8KeyDer::from(
        key_pair.serialize_der(),
    ));

    let server_cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)?;
    Ok(server_cfg)
}

pub fn load_server_config_from_files(
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> Result<ServerConfig, DynError> {
    let cert_file = File::open(cert_path.as_ref())?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
    if certs.is_empty() {
        return Err("no certificates found in certificate PEM file".into());
    }

    let key_file = File::open(key_path.as_ref())?;
    let mut key_reader = BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)?
        .ok_or_else(|| DynError::from("no private key found in key PEM file"))?;

    let server_cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    Ok(server_cfg)
}

pub fn build_https_server_config_from_env() -> Result<Option<ServerConfig>, DynError> {
    let cert_var = std::env::var("XRTAK_HTTPS_CERT").ok();
    let key_var = std::env::var("XRTAK_HTTPS_KEY").ok();
    let self_signed = std::env::var("XRTAK_HTTPS_SELF_SIGNED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if let (Some(cert), Some(key)) = (cert_var, key_var) {
        let cfg = load_server_config_from_files(cert, key)?;
        Ok(Some(cfg))
    } else if self_signed {
        let sans_raw = std::env::var("XRTAK_HTTPS_SANS").unwrap_or_default();
        let sans: Vec<String> = sans_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let cfg = generate_self_signed_server_config(&sans)?;
        Ok(Some(cfg))
    } else {
        Ok(None)
    }
}

pub(crate) async fn spawn_https_server(
    https_addr: SocketAddr,
    server_cfg: ServerConfig,
    state: Arc<GatewayState>,
    conn_limit: Arc<Semaphore>,
) -> Result<SocketAddr, DynError> {
    let listener = TcpListener::bind(https_addr).await?;
    let local_addr = listener.local_addr()?;
    let acceptor = TlsAcceptor::from(Arc::new(server_cfg));

    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("[gateway-https] accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
            };
            let permit = match conn_limit.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => continue,
            };
            let acceptor = acceptor.clone();
            let state_clone = Arc::clone(&state);
            tokio::spawn(async move {
                let _permit = permit;
                let accept_res =
                    tokio::time::timeout(Duration::from_secs(10), acceptor.accept(stream)).await;
                match accept_res {
                    Ok(Ok(mut tls_stream)) => {
                        let _ = crate::handle_http_connection(&mut tls_stream, state_clone).await;
                        use tokio::io::AsyncWriteExt;
                        let _ = tokio::time::timeout(Duration::from_secs(5), tls_stream.shutdown())
                            .await;
                    }
                    Ok(Err(e)) => {
                        eprintln!("[gateway-https] TLS handshake error: {e}");
                    }
                    Err(_) => {
                        eprintln!("[gateway-https] TLS handshake timed out");
                    }
                }
            });
        }
    });

    Ok(local_addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_self_signed_config_with_custom_sans() {
        let extra_sans = vec!["quest3.local".to_string(), "192.168.1.100".to_string()];
        let cfg = generate_self_signed_server_config(&extra_sans);
        assert!(cfg.is_ok());
    }

    #[tokio::test]
    async fn idle_tcp_connection_to_https_listener_times_out() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_cfg = generate_self_signed_server_config(&[]).unwrap();
        let world = Arc::new(std::sync::Mutex::new(xrtak_core::TacticalWorld::new(
            xrtak_core::GeoOrigin::default(),
        )));
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let (status_tx, status_rx) =
            tokio::sync::watch::channel(crate::tak_client::TakClientStatus::default());
        let state = Arc::new(GatewayState {
            world,
            tx,
            tak_client: Arc::new(std::sync::Mutex::new(None)),
            tak_status_tx: status_tx,
            tak_status_rx: status_rx,
            feedback_sock: None,
            feedback_addr: "127.0.0.1:4243".to_string(),
        });
        let conn_limit = Arc::new(Semaphore::new(256));
        let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let actual_addr = spawn_https_server(bind_addr, server_cfg, state, conn_limit.clone())
            .await
            .unwrap();

        // Connect raw TCP without TLS handshake
        let mut stream = tokio::net::TcpStream::connect(actual_addr).await.unwrap();
        // Give spawned accept task a brief moment to acquire permit
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(conn_limit.available_permits(), 255);

        // Keep idle; handshake timeout is 10s. Read until EOF (socket closed by timeout)
        use tokio::io::AsyncReadExt;
        let mut buf = [0u8; 1];
        let read_res = tokio::time::timeout(Duration::from_secs(12), stream.read(&mut buf)).await;
        assert!(read_res.is_ok(), "read timed out waiting for server close");
        assert_eq!(
            read_res.unwrap().unwrap(),
            0,
            "expected EOF on idle disconnect"
        );

        // Wait brief moment for permit drop
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(conn_limit.available_permits(), 256);
    }
}
