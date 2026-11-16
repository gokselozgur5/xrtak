//! TAK server TLS handshake configuration builder (ICD-TAK-TLS-01).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

impl TlsVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tls12 => "TLSv1.2",
            Self::Tls13 => "TLSv1.3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientAuthMode {
    None,
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateFormat {
    Pem,
    Pkcs12,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsCertificate {
    pub format: CertificateFormat,
    pub raw_data: Vec<u8>,
    pub passphrase: Option<String>,
}

impl TlsCertificate {
    pub fn from_pem(data: impl Into<Vec<u8>>) -> Self {
        Self {
            format: CertificateFormat::Pem,
            raw_data: data.into(),
            passphrase: None,
        }
    }

    pub fn from_pkcs12(data: impl Into<Vec<u8>>, passphrase: Option<String>) -> Self {
        Self {
            format: CertificateFormat::Pkcs12,
            raw_data: data.into(),
            passphrase,
        }
    }
    pub fn count_pem_certificates(&self) -> usize {
        if self.format != CertificateFormat::Pem {
            return 0;
        }
        let text = String::from_utf8_lossy(&self.raw_data);
        text.matches("-----BEGIN CERTIFICATE-----").count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakTlsError {
    InvalidHost(String),
    InvalidPort(u16),
    MissingClientCertificate,
    IncompatibleVersion { min: TlsVersion, max: TlsVersion },
    InvalidCertificateData(String),
}

impl std::fmt::Display for TakTlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHost(h) => write!(f, "invalid TAK server host: {h}"),
            Self::InvalidPort(p) => write!(f, "invalid TAK server port: {p}"),
            Self::MissingClientCertificate => {
                write!(f, "mTLS required but no client certificate provided")
            }
            Self::IncompatibleVersion { min, max } => {
                write!(f, "min version ({:?}) exceeds max version ({:?})", min, max)
            }
            Self::InvalidCertificateData(msg) => write!(f, "invalid certificate: {msg}"),
        }
    }
}

impl std::error::Error for TakTlsError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakTlsConfig {
    pub server_host: String,
    pub server_port: u16,
    pub min_version: TlsVersion,
    pub max_version: TlsVersion,
    pub client_auth: ClientAuthMode,
    pub client_cert: Option<TlsCertificate>,
    pub ca_cert: Option<TlsCertificate>,
    pub verify_server_hostname: bool,
    pub sni_hostname: Option<String>,
    pub alpn_protocols: Vec<String>,
    pub handshake_timeout_ms: u64,
    pub cipher_suites: Vec<String>,
}
