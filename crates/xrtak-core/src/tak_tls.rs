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

impl TakTlsConfig {
    pub fn builder(server_host: impl Into<String>, server_port: u16) -> TakTlsConfigBuilder {
        TakTlsConfigBuilder::new(server_host, server_port)
    }

    pub fn is_mtls(&self) -> bool {
        self.client_auth == ClientAuthMode::Required && self.client_cert.is_some()
    }
    pub fn summary(&self) -> String {
        let auth_str = match self.client_auth {
            ClientAuthMode::Required => "mTLS",
            ClientAuthMode::Optional => "TLS-OptionalAuth",
            ClientAuthMode::None => "TLS-ServerAuth",
        };
        format!(
            "{} {}:{} ({}..{}, timeout={}ms)",
            auth_str,
            self.server_host,
            self.server_port,
            self.min_version.as_str(),
            self.max_version.as_str(),
            self.handshake_timeout_ms
        )
    }
    pub fn export_preference_xml(&self) -> String {
        let sni = self.sni_hostname.as_deref().unwrap_or(&self.server_host);
        let client_auth = match self.client_auth {
            ClientAuthMode::Required => "true",
            _ => "false",
        };
        format!(
            "<preference version=\"1\" name=\"cot_stream_tls\">\n\
               <entry key=\"serverHost\">{}</entry>\n\
               <entry key=\"serverPort\">{}</entry>\n\
               <entry key=\"serverSni\">{}</entry>\n\
               <entry key=\"clientAuth\">{}</entry>\n\
               <entry key=\"verifyHost\">{}</entry>\n\
               <entry key=\"timeoutMs\">{}</entry>\n\
             </preference>",
            escape_xml(&self.server_host),
            self.server_port,
            escape_xml(sni),
            client_auth,
            self.verify_server_hostname,
            self.handshake_timeout_ms
        )
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub struct TakTlsConfigBuilder {
    server_host: String,
    server_port: u16,
    min_version: TlsVersion,
    max_version: TlsVersion,
    client_auth: ClientAuthMode,
    client_cert: Option<TlsCertificate>,
    ca_cert: Option<TlsCertificate>,
    verify_server_hostname: bool,
    sni_hostname: Option<String>,
    alpn_protocols: Vec<String>,
    handshake_timeout_ms: u64,
    cipher_suites: Vec<String>,
}

impl TakTlsConfigBuilder {
    pub fn new(server_host: impl Into<String>, server_port: u16) -> Self {
        Self {
            server_host: server_host.into(),
            server_port,
            min_version: TlsVersion::Tls12,
            max_version: TlsVersion::Tls13,
            client_auth: ClientAuthMode::None,
            client_cert: None,
            ca_cert: None,
            verify_server_hostname: true,
            sni_hostname: None,
            alpn_protocols: vec!["cot/1.0".into()],
            handshake_timeout_ms: 10_000,
            cipher_suites: Vec::new(),
        }
    }
    pub fn min_version(mut self, version: TlsVersion) -> Self {
        self.min_version = version;
        self
    }

    pub fn max_version(mut self, version: TlsVersion) -> Self {
        self.max_version = version;
        self
    }

    pub fn client_auth(mut self, mode: ClientAuthMode) -> Self {
        self.client_auth = mode;
        self
    }
    pub fn client_cert(mut self, cert: TlsCertificate) -> Self {
        self.client_cert = Some(cert);
        self
    }

    pub fn ca_cert(mut self, cert: TlsCertificate) -> Self {
        self.ca_cert = Some(cert);
        self
    }
    pub fn verify_server_hostname(mut self, verify: bool) -> Self {
        self.verify_server_hostname = verify;
        self
    }

    pub fn sni_hostname(mut self, sni: impl Into<String>) -> Self {
        self.sni_hostname = Some(sni.into());
        self
    }

    pub fn alpn_protocols(mut self, protocols: Vec<String>) -> Self {
        self.alpn_protocols = protocols;
        self
    }
    pub fn handshake_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.handshake_timeout_ms = timeout_ms;
        self
    }

    pub fn cipher_suites(mut self, suites: Vec<String>) -> Self {
        self.cipher_suites = suites;
        self
    }
    pub fn build(self) -> Result<TakTlsConfig, TakTlsError> {
        let host = self.server_host.trim();
        if host.is_empty() {
            return Err(TakTlsError::InvalidHost("empty hostname".into()));
        }
        if self.server_port == 0 {
            return Err(TakTlsError::InvalidPort(0));
        }
        if self.min_version > self.max_version {
            return Err(TakTlsError::IncompatibleVersion {
                min: self.min_version,
                max: self.max_version,
            });
        }
        if self.client_auth == ClientAuthMode::Required && self.client_cert.is_none() {
            return Err(TakTlsError::MissingClientCertificate);
        }
        Ok(TakTlsConfig {
            server_host: host.to_string(),
            server_port: self.server_port,
            min_version: self.min_version,
            max_version: self.max_version,
            client_auth: self.client_auth,
            client_cert: self.client_cert,
            ca_cert: self.ca_cert,
            verify_server_hostname: self.verify_server_hostname,
            sni_hostname: self.sni_hostname,
            alpn_protocols: self.alpn_protocols,
            handshake_timeout_ms: self.handshake_timeout_ms,
            cipher_suites: self.cipher_suites,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_builder() {
        let config = TakTlsConfig::builder("tak.domain.mil", 8089)
            .build()
            .unwrap();

        assert_eq!(config.server_host, "tak.domain.mil");
        assert_eq!(config.server_port, 8089);
        assert_eq!(config.min_version, TlsVersion::Tls12);
        assert_eq!(config.max_version, TlsVersion::Tls13);
        assert_eq!(config.client_auth, ClientAuthMode::None);
        assert!(config.verify_server_hostname);
        assert!(!config.is_mtls());
    }

    #[test]
    fn test_mtls_valid_configuration() {
        let pem_data = b"-----BEGIN CERTIFICATE-----\nMIIB...\n-----END CERTIFICATE-----\n";
        let cert = TlsCertificate::from_pem(pem_data.to_vec());
        assert_eq!(cert.count_pem_certificates(), 1);

        let config = TakTlsConfig::builder("tak.domain.mil", 8089)
            .client_auth(ClientAuthMode::Required)
            .client_cert(cert)
            .sni_hostname("custom-sni.mil")
            .build()
            .unwrap();

        assert!(config.is_mtls());
        assert_eq!(config.sni_hostname.as_deref(), Some("custom-sni.mil"));
    }

    #[test]
    fn test_mtls_missing_client_cert_fails() {
        let err = TakTlsConfig::builder("tak.domain.mil", 8089)
            .client_auth(ClientAuthMode::Required)
            .build()
            .unwrap_err();

        assert_eq!(err, TakTlsError::MissingClientCertificate);
    }

    #[test]
    fn test_invalid_host_and_port() {
        let err_host = TakTlsConfig::builder("   ", 8089).build().unwrap_err();
        assert!(matches!(err_host, TakTlsError::InvalidHost(_)));

        let err_port = TakTlsConfig::builder("tak.domain.mil", 0)
            .build()
            .unwrap_err();
        assert_eq!(err_port, TakTlsError::InvalidPort(0));
    }

    #[test]
    fn test_incompatible_tls_versions() {
        let err = TakTlsConfig::builder("tak.domain.mil", 8089)
            .min_version(TlsVersion::Tls13)
            .max_version(TlsVersion::Tls12)
            .build()
            .unwrap_err();

        assert_eq!(
            err,
            TakTlsError::IncompatibleVersion {
                min: TlsVersion::Tls13,
                max: TlsVersion::Tls12
            }
        );
    }

    #[test]
    fn test_export_preference_xml_and_summary() {
        let config = TakTlsConfig::builder("tak.domain.mil", 8089)
            .handshake_timeout_ms(5000)
            .build()
            .unwrap();

        let xml = config.export_preference_xml();
        assert!(xml.contains("<preference version=\"1\" name=\"cot_stream_tls\">"));
        assert!(xml.contains("<entry key=\"serverHost\">tak.domain.mil</entry>"));
        assert!(xml.contains("<entry key=\"serverPort\">8089</entry>"));
        assert!(xml.contains("<entry key=\"timeoutMs\">5000</entry>"));

        let summary = config.summary();
        assert!(summary.contains("TLS-ServerAuth tak.domain.mil:8089"));
        assert!(summary.contains("timeout=5000ms"));
    }
}
