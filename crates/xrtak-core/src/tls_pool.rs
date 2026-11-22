//! TAK server TLS connection pool and mutual certificate authenticator (ICD-TAK-TLS-POOL-01).
//!
//! No TLS is performed here: connections are simulated and start unverified.

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
        if h.parse::<std::net::IpAddr>().is_err() {
            if let Some(suffix) = p.strip_prefix("*.") {
                if let Some((_, host_domain)) = h.split_once('.') {
                    return host_domain == suffix;
                }
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

        let host_candidate = if expected_host.starts_with('[') {
            expected_host
                .split(']')
                .next()
                .map(|s| s.trim_start_matches('['))
                .unwrap_or(expected_host)
        } else {
            expected_host.split(':').next().unwrap_or(expected_host)
        };
        let host_matched = if cert.subject_alt_names.is_empty() {
            Self::matches_host_or_pattern(&cert.subject_cn, host_candidate)
        } else {
            cert.subject_alt_names
                .iter()
                .any(|san| Self::matches_host_or_pattern(san, host_candidate))
        };

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
    }

    pub fn config(&self) -> &TlsPoolConfig {
        &self.config
    }

    pub fn add_endpoint(&mut self, endpoint: impl Into<String>) {
        let ep = endpoint.into();
        if !self.config.server_endpoints.contains(&ep) {
            self.config.server_endpoints.push(ep);
        }
    }

    pub fn remove_endpoint(&mut self, endpoint: &str) {
        self.config.server_endpoints.retain(|e| e != endpoint);
        for conn in &mut self.connections {
            if conn.endpoint == endpoint {
                if conn.state == ConnectionState::Idle {
                    conn.state = ConnectionState::Closed;
                } else if conn.state == ConnectionState::Active {
                    conn.state = ConnectionState::Draining;
                }
            }
        }
    }

    pub fn acquire_connection(
        &mut self,
        endpoint: &str,
        current_time_ms: u64,
    ) -> Result<u64, PoolError> {
        let idle_idx = self.connections.iter().position(|c| {
            c.endpoint == endpoint
                && c.state == ConnectionState::Idle
                && !c.is_expired(current_time_ms, self.config.connection_lifetime_ms)
        });

        if let Some(idx) = idle_idx {
            let conn = &mut self.connections[idx];
            conn.state = ConnectionState::Active;
            conn.last_active_epoch_ms = current_time_ms;
            self.total_acquired += 1;
            return Ok(conn.id);
        }

        let total_active_or_idle = self
            .connections
            .iter()
            .filter(|c| c.state.is_alive())
            .count();
        if total_active_or_idle >= self.config.max_total_connections {
            return Err(PoolError::PoolExhausted);
        }

        let ep_count = self
            .connections
            .iter()
            .filter(|c| c.endpoint == endpoint && c.state.is_alive())
            .count();
        if ep_count >= self.config.max_connections_per_endpoint {
            return Err(PoolError::EndpointExhausted(endpoint.to_string()));
        }

        let new_id = self.next_conn_id;
        self.next_conn_id += 1;

        let mut new_conn = TlsConnection::new(new_id, endpoint, current_time_ms);
        new_conn.state = ConnectionState::Active;

        self.connections.push(new_conn);
        self.total_created += 1;
        self.total_acquired += 1;

        Ok(new_id)
    }

    pub fn release_connection(
        &mut self,
        conn_id: u64,
        current_time_ms: u64,
    ) -> Result<(), PoolError> {
        let lifetime = self.config.connection_lifetime_ms;
        let conn = self
            .connections
            .iter_mut()
            .find(|c| c.id == conn_id)
            .ok_or(PoolError::ConnectionNotFound(conn_id))?;

        if conn.state != ConnectionState::Active && conn.state != ConnectionState::Draining {
            return Err(PoolError::InvalidState(format!(
                "cannot release connection in state {}",
                conn.state
            )));
        }

        if conn.state == ConnectionState::Draining || conn.is_expired(current_time_ms, lifetime) {
            conn.state = ConnectionState::Closed;
        } else {
            conn.state = ConnectionState::Idle;
            conn.last_active_epoch_ms = current_time_ms;
        }

        self.total_released += 1;
        Ok(())
    }

    pub fn record_io(
        &mut self,
        conn_id: u64,
        sent: u64,
        received: u64,
        current_time_ms: u64,
    ) -> Result<(), PoolError> {
        let conn = self
            .connections
            .iter_mut()
            .find(|c| c.id == conn_id)
            .ok_or(PoolError::ConnectionNotFound(conn_id))?;

        if !conn.state.is_alive() {
            return Err(PoolError::InvalidState(format!(
                "cannot record I/O on dead connection in state {}",
                conn.state
            )));
        }

        conn.bytes_sent = conn.bytes_sent.saturating_add(sent);
        conn.bytes_received = conn.bytes_received.saturating_add(received);
        conn.last_active_epoch_ms = current_time_ms;
        Ok(())
    }

    pub fn mark_connection_failed(
        &mut self,
        conn_id: u64,
        reason: impl Into<String>,
    ) -> Result<(), PoolError> {
        let conn = self
            .connections
            .iter_mut()
            .find(|c| c.id == conn_id)
            .ok_or(PoolError::ConnectionNotFound(conn_id))?;

        conn.state = ConnectionState::Failed;
        conn.failure_reason = Some(reason.into());
        Ok(())
    }

    pub fn record_auth_failure(&mut self) {
        self.total_auth_failures += 1;
    }

    pub fn prune_idle_and_expired(&mut self, current_time_ms: u64) -> usize {
        let idle_timeout = self.config.idle_timeout_ms;
        let lifetime = self.config.connection_lifetime_ms;
        let min_idle = self.config.min_idle_connections;

        let mut idle_per_endpoint = std::collections::HashMap::<String, usize>::new();
        for conn in self.connections.iter() {
            if conn.state == ConnectionState::Idle && !conn.is_expired(current_time_ms, lifetime) {
                *idle_per_endpoint.entry(conn.endpoint.clone()).or_insert(0) += 1;
            }
        }

        let mut pruned = 0;
        for conn in self.connections.iter_mut() {
            if !conn.state.is_alive() {
                continue;
            }

            if conn.is_expired(current_time_ms, lifetime) {
                if conn.state == ConnectionState::Active {
                    conn.state = ConnectionState::Draining;
                } else {
                    conn.state = ConnectionState::Closed;
                    pruned += 1;
                }
                continue;
            }

            if conn.is_idle_timed_out(current_time_ms, idle_timeout) {
                let current_idle_count =
                    idle_per_endpoint.get(&conn.endpoint).copied().unwrap_or(0);
                if current_idle_count > min_idle {
                    conn.state = ConnectionState::Closed;
                    idle_per_endpoint
                        .insert(conn.endpoint.clone(), current_idle_count.saturating_sub(1));
                    pruned += 1;
                }
            }
        }

        self.connections.retain(|c| c.state.is_alive());
        self.total_pruned += pruned as u64;
        pruned
    }

    pub fn run_health_checks(&mut self, current_time_ms: u64) -> HealthCheckSummary {
        let mut summary = HealthCheckSummary::default();

        for conn in self.connections.iter_mut() {
            if !conn.state.is_alive() {
                continue;
            }
            summary.total_checked += 1;

            if conn.is_expired(current_time_ms, self.config.connection_lifetime_ms) {
                if conn.state == ConnectionState::Active {
                    conn.state = ConnectionState::Draining;
                } else {
                    conn.state = ConnectionState::Closed;
                }
                conn.failure_reason = Some("connection lifetime expired".into());
                summary.failed_count += 1;
            } else if conn.failure_reason.is_some() {
                summary.degraded_count += 1;
            } else {
                conn.health_checks_passed += 1;
                conn.last_health_check_epoch_ms = current_time_ms;
                summary.healthy_count += 1;
            }
        }

        summary
    }

    pub fn get_connection(&self, conn_id: u64) -> Option<&TlsConnection> {
        self.connections.iter().find(|c| c.id == conn_id)
    }

    pub fn pool_metrics(&self) -> PoolMetrics {
        let mut metrics = PoolMetrics {
            total_connections: self.connections.len(),
            total_created: self.total_created,
            total_acquired: self.total_acquired,
            total_released: self.total_released,
            total_pruned: self.total_pruned,
            total_auth_failures: self.total_auth_failures,
            ..Default::default()
        };

        for conn in &self.connections {
            match conn.state {
                ConnectionState::Active => metrics.active_connections += 1,
                ConnectionState::Idle => metrics.idle_connections += 1,
                ConnectionState::Failed => metrics.failed_connections += 1,
                _ => {}
            }
            metrics.total_bytes_sent += conn.bytes_sent;
            metrics.total_bytes_received += conn.bytes_received;
        }

        metrics
    }

    pub fn close_all(&mut self) {
        for conn in &mut self.connections {
            conn.state = ConnectionState::Closed;
        }
        self.connections.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_defaults_and_builder() {
        let config = TlsPoolConfig::new()
            .with_endpoint("10.0.0.1:8089")
            .with_max_total_connections(8)
            .with_idle_timeout_ms(15_000)
            .with_lifetime_ms(120_000)
            .with_allowed_cn("takserver.local")
            .with_client_identity("user1@agency", "AA:BB:CC:DD");

        assert_eq!(config.server_endpoints.len(), 1);
        assert_eq!(config.max_total_connections, 8);
        assert_eq!(config.idle_timeout_ms, 15_000);
        assert_eq!(config.connection_lifetime_ms, 120_000);
        assert_eq!(config.client_cert_cn.as_deref(), Some("user1@agency"));
    }

    #[test]
    fn test_connection_lifecycle_acquire_and_release() {
        let config = TlsPoolConfig::new()
            .with_endpoint("tak.example.com:8089")
            .with_client_identity("client1", "FINGERPRINT");
        let mut pool = TlsConnectionPool::new(config);

        let id = pool
            .acquire_connection("tak.example.com:8089", 1000)
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(pool.pool_metrics().active_connections, 1);

        pool.record_io(id, 128, 256, 1100).unwrap();
        let conn = pool.get_connection(id).unwrap();
        assert_eq!(conn.bytes_sent, 128);
        assert_eq!(conn.bytes_received, 256);

        pool.release_connection(id, 1200).unwrap();
        assert_eq!(pool.pool_metrics().active_connections, 0);
        assert_eq!(pool.pool_metrics().idle_connections, 1);

        let id2 = pool
            .acquire_connection("tak.example.com:8089", 1300)
            .unwrap();
        assert_eq!(id2, 1);
        assert_eq!(pool.pool_metrics().active_connections, 1);
    }

    #[test]
    fn test_pool_limits_and_exhaustion() {
        let mut config = TlsPoolConfig::new();
        config.max_connections_per_endpoint = 2;
        config.max_total_connections = 3;
        let mut pool = TlsConnectionPool::new(config);

        let id1 = pool.acquire_connection("server-a:8089", 100).unwrap();
        let id2 = pool.acquire_connection("server-a:8089", 100).unwrap();
        assert_ne!(id1, id2);

        let err1 = pool.acquire_connection("server-a:8089", 100).unwrap_err();
        assert!(matches!(err1, PoolError::EndpointExhausted(_)));

        let _id3 = pool.acquire_connection("server-b:8089", 100).unwrap();
        let err2 = pool.acquire_connection("server-b:8089", 100).unwrap_err();
        assert_eq!(err2, PoolError::PoolExhausted);
    }

    #[test]
    fn test_mutual_auth_peer_validation_success_and_wildcard() {
        let config = TlsPoolConfig::new()
            .with_client_identity("user", "FINGERPRINT")
            .with_pinned_fingerprint("AA:BB:CC:DD")
            .with_allowed_cn("tak.example.com");

        let cert = PeerCertificateInfo {
            subject_cn: "tak.example.com".into(),
            subject_alt_names: vec!["tak.example.com".into(), "*.internal.tak".into()],
            issuer: "Root TAK CA".into(),
            not_before_epoch_ms: 500,
            not_after_epoch_ms: 5000,
            sha256_fingerprint: "aa:bb:cc:dd".into(),
            is_ca: false,
        };

        assert!(MutualCertAuthenticator::validate_peer(
            &cert,
            "tak.example.com:8089",
            1000,
            &config
        )
        .is_ok());
        assert!(MutualCertAuthenticator::validate_peer(
            &cert,
            "node1.internal.tak:8089",
            1000,
            &config
        )
        .is_ok());
    }

    #[test]
    fn test_mutual_auth_rejections() {
        let config = TlsPoolConfig::new()
            .with_client_identity("user", "FP")
            .with_pinned_fingerprint("11:22:33:44");

        let cert = PeerCertificateInfo {
            subject_cn: "tak.example.com".into(),
            subject_alt_names: vec![],
            issuer: "TAK CA".into(),
            not_before_epoch_ms: 1000,
            not_after_epoch_ms: 2000,
            sha256_fingerprint: "FF:EE:DD:CC".into(),
            is_ca: false,
        };

        let err_expired =
            MutualCertAuthenticator::validate_peer(&cert, "tak.example.com", 2500, &config)
                .unwrap_err();
        assert!(matches!(err_expired, AuthError::CertificateExpired { .. }));

        let err_fp =
            MutualCertAuthenticator::validate_peer(&cert, "tak.example.com", 1500, &config)
                .unwrap_err();
        assert!(matches!(err_fp, AuthError::FingerprintMismatch { .. }));

        let err_host =
            MutualCertAuthenticator::validate_peer(&cert, "other.server.com", 1500, &config)
                .unwrap_err();
        assert!(matches!(err_host, AuthError::HostnameMismatch { .. }));
    }

    #[test]
    fn test_idle_and_expired_pruning() {
        let mut config = TlsPoolConfig::new();
        config.min_idle_connections = 0;
        config.idle_timeout_ms = 500;
        config.connection_lifetime_ms = 2000;
        let mut pool = TlsConnectionPool::new(config);

        let id1 = pool.acquire_connection("tak.local:8089", 100).unwrap();
        let id2 = pool.acquire_connection("tak.local:8089", 100).unwrap();
        pool.release_connection(id1, 200).unwrap();
        pool.release_connection(id2, 200).unwrap();

        assert_eq!(pool.prune_idle_and_expired(400), 0);
        assert_eq!(pool.prune_idle_and_expired(800), 2);
        assert_eq!(pool.pool_metrics().total_connections, 0);
    }

    #[test]
    fn test_health_checks_and_metrics_summary() {
        let mut config = TlsPoolConfig::new();
        config.connection_lifetime_ms = 1000;
        let mut pool = TlsConnectionPool::new(config);

        let id = pool.acquire_connection("ep:8089", 100).unwrap();
        let summary1 = pool.run_health_checks(200);
        assert_eq!(summary1.healthy_count, 1);
        assert_eq!(summary1.failed_count, 0);

        pool.release_connection(id, 300).unwrap();
        let summary2 = pool.run_health_checks(1200);
        assert_eq!(summary2.failed_count, 1);
    }
}
