//! TAK server TLS connection pool and mutual certificate authenticator (ICD-TAK-TLS-POOL-01).

use serde::{Deserialize, Serialize};

/// Error types occurring during connection pool operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    /// The pool has reached its maximum global connection capacity.
    PoolExhausted,
    /// An endpoint has reached its maximum per-endpoint connection capacity.
    EndpointExhausted(String),
    /// The requested connection ID was not found in the pool.
    ConnectionNotFound(u64),
    /// Connection is in an invalid state for the requested operation.
    InvalidState(String),
    /// Mutual TLS authentication failed for the connection.
    AuthenticationFailed(String),
    /// Operation timed out.
    Timeout(String),
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PoolExhausted => write!(f, "connection pool capacity exhausted"),
            Self::EndpointExhausted(ep) => write!(f, "endpoint capacity exhausted for: {ep}"),
            Self::ConnectionNotFound(id) => write!(f, "connection id {id} not found"),
            Self::InvalidState(msg) => write!(f, "invalid connection state: {msg}"),
            Self::AuthenticationFailed(msg) => write!(f, "mutual authentication failed: {msg}"),
            Self::Timeout(msg) => write!(f, "pool operation timed out: {msg}"),
        }
    }
}

impl std::error::Error for PoolError {}

/// Configuration for the TAK server TLS connection pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TlsPoolConfig {
    /// Configured TAK server endpoints (host:port).
    pub server_endpoints: Vec<String>,
    /// Maximum simultaneous connections permitted per endpoint.
    pub max_connections_per_endpoint: usize,
    /// Maximum global connections allowed across all endpoints.
    pub max_total_connections: usize,
    /// Minimum idle connections to maintain ready in the pool.
    pub min_idle_connections: usize,
    /// Maximum duration (ms) an idle connection may remain in pool before eviction.
    pub idle_timeout_ms: u64,
    /// Maximum absolute lifetime (ms) of a connection regardless of activity.
    pub connection_lifetime_ms: u64,
    /// Maximum duration (ms) allowed for TLS handshake completion.
    pub handshake_timeout_ms: u64,
    /// Interval (ms) between periodic connection health checks.
    pub health_check_interval_ms: u64,
    /// Whether mutual TLS certificate verification is enforced.
    pub enable_mtls: bool,
    /// Whitelist of acceptable peer certificate Common Names. Empty means any trusted CN.
    pub allowed_common_names: Vec<String>,
    /// Pinned SHA-256 certificate fingerprints for server validation.
    pub pinned_sha256_fingerprints: Vec<String>,
    /// Client certificate identity Common Name.
    pub client_cert_cn: Option<String>,
    /// Client certificate SHA-256 fingerprint.
    pub client_cert_fingerprint: Option<String>,
    /// Negotiated ALPN application protocols.
    pub alpn_protocols: Vec<String>,
}

impl Default for TlsPoolConfig {
    fn default() -> Self {
        Self {
            server_endpoints: Vec::new(),
            max_connections_per_endpoint: 4,
            max_total_connections: 16,
            min_idle_connections: 1,
            idle_timeout_ms: 60_000,
            connection_lifetime_ms: 300_000,
            handshake_timeout_ms: 10_000,
            health_check_interval_ms: 30_000,
            enable_mtls: true,
            allowed_common_names: Vec::new(),
            pinned_sha256_fingerprints: Vec::new(),
            client_cert_cn: None,
            client_cert_fingerprint: None,
            alpn_protocols: vec!["tak-cot".into(), "cot-xml".into()],
        }
    }
}

impl TlsPoolConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.server_endpoints.push(endpoint.into());
        self
    }

    pub fn with_max_total_connections(mut self, max: usize) -> Self {
        self.max_total_connections = max;
        self
    }

    pub fn with_idle_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.idle_timeout_ms = timeout_ms;
        self
    }

    pub fn with_lifetime_ms(mut self, lifetime_ms: u64) -> Self {
        self.connection_lifetime_ms = lifetime_ms;
        self
    }

    pub fn with_pinned_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.pinned_sha256_fingerprints.push(fingerprint.into());
        self
    }

    pub fn with_allowed_cn(mut self, cn: impl Into<String>) -> Self {
        self.allowed_common_names.push(cn.into());
        self
    }

    pub fn with_client_identity(
        mut self,
        cn: impl Into<String>,
        fingerprint: impl Into<String>,
    ) -> Self {
        self.client_cert_cn = Some(cn.into());
        self.client_cert_fingerprint = Some(fingerprint.into());
        self
    }
}

/// Lifecycle state of a pooled TLS connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Socket connected, TLS handshake initiated.
    Connecting,
    /// TLS handshake completed, ready for activation.
    HandshakeComplete,
    /// Connection currently leased by an active caller.
    Active,
    /// Connection idle in pool waiting for lease.
    Idle,
    /// Connection marked for graceful closure after current operations finish.
    Draining,
    /// Connection permanently closed.
    Closed,
    /// Connection experienced fatal error or failed validation.
    Failed,
}

impl ConnectionState {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self, Self::Closed | Self::Failed)
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connecting => write!(f, "CONNECTING"),
            Self::HandshakeComplete => write!(f, "HANDSHAKE_COMPLETE"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Idle => write!(f, "IDLE"),
            Self::Draining => write!(f, "DRAINING"),
            Self::Closed => write!(f, "CLOSED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

/// Metadata and state for an individual pooled TLS connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TlsConnection {
    pub id: u64,
    pub endpoint: String,
    pub state: ConnectionState,
    pub created_at_epoch_ms: u64,
    pub last_active_epoch_ms: u64,
    pub last_health_check_epoch_ms: u64,
    pub tls_version: String,
    pub cipher_suite: String,
    pub peer_identity: Option<String>,
    pub is_mtls_verified: bool,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub health_checks_passed: u32,
    pub failure_reason: Option<String>,
}

impl TlsConnection {
    pub fn new(id: u64, endpoint: impl Into<String>, created_at_epoch_ms: u64) -> Self {
        Self {
            id,
            endpoint: endpoint.into(),
            state: ConnectionState::Idle,
            created_at_epoch_ms,
            last_active_epoch_ms: created_at_epoch_ms,
            last_health_check_epoch_ms: created_at_epoch_ms,
            tls_version: "TLSv1.3".into(),
            cipher_suite: "TLS_AES_256_GCM_SHA384".into(),
            peer_identity: None,
            is_mtls_verified: false,
            bytes_sent: 0,
            bytes_received: 0,
            health_checks_passed: 0,
            failure_reason: None,
        }
    }

    pub fn is_expired(&self, current_time_ms: u64, lifetime_ms: u64) -> bool {
        current_time_ms.saturating_sub(self.created_at_epoch_ms) >= lifetime_ms
    }

    pub fn is_idle_timed_out(&self, current_time_ms: u64, idle_timeout_ms: u64) -> bool {
        self.state == ConnectionState::Idle
            && current_time_ms.saturating_sub(self.last_active_epoch_ms) >= idle_timeout_ms
    }
}

/// Peer certificate details extracted during mutual TLS handshake.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerCertificateInfo {
    pub subject_cn: String,
    pub subject_alt_names: Vec<String>,
    pub issuer: String,
    pub not_before_epoch_ms: u64,
    pub not_after_epoch_ms: u64,
    pub sha256_fingerprint: String,
    pub is_ca: bool,
}

/// Errors raised during mutual certificate authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    CertificateExpired {
        expired_at: u64,
        current_time: u64,
    },
    CertificateNotYetValid {
        valid_from: u64,
        current_time: u64,
    },
    HostnameMismatch {
        expected: String,
        found_cn: String,
        sans: Vec<String>,
    },
    FingerprintMismatch {
        expected: Vec<String>,
        actual: String,
    },
    DisallowedCommonName {
        cn: String,
        allowed: Vec<String>,
    },
    MissingClientCredentials,
    InvalidCertificateFormat(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CertificateExpired {
                expired_at,
                current_time,
            } => {
                write!(
                    f,
                    "certificate expired at {expired_at} (current {current_time})"
                )
            }
            Self::CertificateNotYetValid {
                valid_from,
                current_time,
            } => {
                write!(
                    f,
                    "certificate not yet valid until {valid_from} (current {current_time})"
                )
            }
            Self::HostnameMismatch {
                expected,
                found_cn,
                sans,
            } => {
                write!(
                    f,
                    "hostname mismatch: expected {expected}, found CN={found_cn}, SANs={sans:?}"
                )
            }
            Self::FingerprintMismatch { expected, actual } => {
                write!(f, "fingerprint {actual} not in pinned list: {expected:?}")
            }
            Self::DisallowedCommonName { cn, allowed } => {
                write!(f, "CN {cn} not in allowed list: {allowed:?}")
            }
            Self::MissingClientCredentials => {
                write!(f, "mTLS enabled but no client credentials provided")
            }
            Self::InvalidCertificateFormat(err) => {
                write!(f, "invalid certificate format: {err}")
            }
        }
    }
}

impl std::error::Error for AuthError {}

/// Mutual certificate authenticator performing RFC 5280 and ICD-TAK-TLS-01 validations.
pub struct MutualCertAuthenticator;

impl MutualCertAuthenticator {
    fn normalize_fingerprint(fp: &str) -> String {
        fp.replace([':', ' ', '-'], "").to_uppercase()
    }

    fn matches_host_or_pattern(pattern: &str, host: &str) -> bool {
        let p = pattern.to_lowercase();
        let h = host.to_lowercase();
        if p == h {
            return true;
        }
        if let Some(suffix) = p.strip_prefix("*.") {
            if let Some((_, host_domain)) = h.split_once('.') {
                return host_domain == suffix;
            }
        }
        false
    }

    pub fn validate_peer(
        cert: &PeerCertificateInfo,
        expected_host: &str,
        current_time_ms: u64,
        config: &TlsPoolConfig,
    ) -> Result<(), AuthError> {
        if current_time_ms < cert.not_before_epoch_ms {
            return Err(AuthError::CertificateNotYetValid {
                valid_from: cert.not_before_epoch_ms,
                current_time: current_time_ms,
            });
        }
        if current_time_ms > cert.not_after_epoch_ms {
            return Err(AuthError::CertificateExpired {
                expired_at: cert.not_after_epoch_ms,
                current_time: current_time_ms,
            });
        }

        let host_candidate = expected_host.split(':').next().unwrap_or(expected_host);
        let host_matched = Self::matches_host_or_pattern(&cert.subject_cn, host_candidate)
            || cert
                .subject_alt_names
                .iter()
                .any(|san| Self::matches_host_or_pattern(san, host_candidate));

        if !host_matched {
            return Err(AuthError::HostnameMismatch {
                expected: host_candidate.to_string(),
                found_cn: cert.subject_cn.clone(),
                sans: cert.subject_alt_names.clone(),
            });
        }

        if !config.pinned_sha256_fingerprints.is_empty() {
            let normalized_actual = Self::normalize_fingerprint(&cert.sha256_fingerprint);
            let matched = config
                .pinned_sha256_fingerprints
                .iter()
                .any(|pinned| Self::normalize_fingerprint(pinned) == normalized_actual);
            if !matched {
                return Err(AuthError::FingerprintMismatch {
                    expected: config.pinned_sha256_fingerprints.clone(),
                    actual: cert.sha256_fingerprint.clone(),
                });
            }
        }

        if !config.allowed_common_names.is_empty()
            && !config
                .allowed_common_names
                .iter()
                .any(|cn| cn == &cert.subject_cn)
        {
            return Err(AuthError::DisallowedCommonName {
                cn: cert.subject_cn.clone(),
                allowed: config.allowed_common_names.clone(),
            });
        }

        if config.enable_mtls && config.client_cert_cn.is_none() {
            return Err(AuthError::MissingClientCredentials);
        }

        Ok(())
    }
}

/// Summary of a batch health check run.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HealthCheckSummary {
    pub total_checked: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub failed_count: usize,
}

/// Operational statistics of the connection pool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub failed_connections: usize,
    pub total_created: u64,
    pub total_acquired: u64,
    pub total_released: u64,
    pub total_pruned: u64,
    pub total_auth_failures: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

/// Managed pool of mutual-TLS connections to TAK server endpoints.
pub struct TlsConnectionPool {
    config: TlsPoolConfig,
    next_conn_id: u64,
    connections: Vec<TlsConnection>,
    total_created: u64,
    total_acquired: u64,
    total_released: u64,
    total_pruned: u64,
    total_auth_failures: u64,
}

impl TlsConnectionPool {
    pub fn new(config: TlsPoolConfig) -> Self {
        Self {
            config,
            next_conn_id: 1,
            connections: Vec::new(),
            total_created: 0,
            total_acquired: 0,
            total_released: 0,
            total_pruned: 0,
            total_auth_failures: 0,
        }
