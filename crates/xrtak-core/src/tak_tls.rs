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
