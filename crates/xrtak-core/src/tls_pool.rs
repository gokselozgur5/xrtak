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

