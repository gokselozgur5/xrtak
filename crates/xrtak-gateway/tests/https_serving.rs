use rustls::pki_types::ServerName;
use rustls::ClientConfig;
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

fn start_gateway_https() -> (Child, u16, u16) {
    let http_port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let https_port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    let http_addr = format!("127.0.0.1:{http_port}");
    let https_addr = format!("127.0.0.1:{https_port}");
    let udp_addr = format!("127.0.0.1:{http_port}");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.env("XRTAK_HTTP_ADDR", &http_addr)
        .env("XRTAK_HTTPS_ADDR", &https_addr)
        .env("XRTAK_UDP_ADDR", &udp_addr)
        .env("XRTAK_HTTPS_SELF_SIGNED", "1");

    let child = cmd.spawn().expect("failed to spawn xrtak-gateway");

    for _ in 0..50 {
        if std::net::TcpStream::connect(&https_addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(150));
    (child, http_port, https_port)
}

fn stop_child(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[derive(Debug)]
struct NoCertVerifier;

impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[tokio::test]
async fn loopback_https_get_api_world_returns_200_json() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (child, _http_port, https_port) = start_gateway_https();

    let client_cfg = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(client_cfg));
    let server_name = ServerName::try_from("localhost".to_string()).unwrap();

    let tcp_stream = TcpStream::connect(("127.0.0.1", https_port))
        .await
        .expect("tcp connect to https port");

    let mut tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .expect("tls handshake");

    let req = "GET /api/world HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    tls_stream
        .write_all(req.as_bytes())
        .await
        .expect("write request");

    let mut response_buf = Vec::new();
    tls_stream
        .read_to_end(&mut response_buf)
        .await
        .expect("read response");

    stop_child(child);

    let response_str = String::from_utf8_lossy(&response_buf);
    assert!(
        response_str.starts_with("HTTP/1.1 200 OK"),
        "expected 200 OK, got: {response_str}"
    );
    assert!(
        response_str.contains("Content-Type: application/json"),
        "expected json content type"
    );
    assert!(
        response_str.contains("\"terrain_name\""),
        "expected world payload containing terrain_name"
    );
}
