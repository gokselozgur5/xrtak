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
