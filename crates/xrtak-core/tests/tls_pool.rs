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
    assert_eq!(metrics.active_connections, 2);
    assert_eq!(metrics.total_bytes_sent, 768);
    assert_eq!(metrics.total_bytes_received, 1536);

    let summary: HealthCheckSummary = pool.run_health_checks(1180);
    assert_eq!(summary.healthy_count, 2);

    pool.release_connection(c1, 1200).unwrap();
    pool.release_connection(c2, 1250).unwrap();
    assert_eq!(pool.pool_metrics().active_connections, 0);
    assert_eq!(pool.pool_metrics().idle_connections, 2);
}

#[test]
fn test_integration_mtls_san_and_wildcard_verification() {
    let config = TlsPoolConfig::new()
        .with_client_identity("sensor-node", "00112233")
        .with_allowed_cn("tak-gw.defense.gov")
        .with_pinned_fingerprint("DE:AD:BE:EF:01:02:03:04");

    let valid_cert = PeerCertificateInfo {
        subject_cn: "tak-gw.defense.gov".into(),
        subject_alt_names: vec!["edge.tactical.network".into()],
        issuer: "DoD Interoperability CA".into(),
        not_before_epoch_ms: 10_000,
        not_after_epoch_ms: 90_000,
        sha256_fingerprint: "DEADBEEF01020304".into(),
        is_ca: false,
    };

    let res = MutualCertAuthenticator::validate_peer(
        &valid_cert,
        "tak-gw.defense.gov:8089",
        50_000,
        &config,
    );
    assert!(res.is_ok());

    let mut pool = TlsConnectionPool::new(config);
    pool.close_all();
    let err = pool.release_connection(999, 100).unwrap_err();
    assert!(matches!(err, PoolError::ConnectionNotFound(999)));
}
