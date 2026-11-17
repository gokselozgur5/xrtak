//! Integration tests for TAK server TLS connection pool (XR-078).

use xrtak_core::tls_pool::{
    HealthCheckSummary, MutualCertAuthenticator, PeerCertificateInfo, PoolError, TlsConnectionPool,
    TlsPoolConfig,
};

#[test]
fn test_integration_tls_pool_multiserver_failover() {
    let config = TlsPoolConfig::new()
        .with_endpoint("tak1.local:8089")
        .with_endpoint("tak2.local:8089")
        .with_client_identity("client-xr", "A1B2C3D4")
        .with_max_total_connections(6);
    let mut pool = TlsConnectionPool::new(config);

    let c1 = pool.acquire_connection("tak1.local:8089", 1000).unwrap();
    let c2 = pool.acquire_connection("tak2.local:8089", 1050).unwrap();
    assert_ne!(c1, c2);

    pool.record_io(c1, 512, 1024, 1100).unwrap();
    pool.record_io(c2, 256, 512, 1150).unwrap();

    let metrics = pool.pool_metrics();
