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
