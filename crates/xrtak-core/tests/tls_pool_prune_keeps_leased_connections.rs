use xrtak_core::tls_pool::{TlsConnectionPool, TlsPoolConfig};

#[test]
fn prune_does_not_drop_a_connection_that_is_still_leased() {
    let mut config = TlsPoolConfig::new();
    config.connection_lifetime_ms = 1_000;
    let mut pool = TlsConnectionPool::new(config);

    let id = pool.acquire_connection("tak.example.com:8089", 0).unwrap();
    pool.prune_idle_and_expired(2_000);

    assert!(
        pool.get_connection(id).is_some(),
        "leased connection removed while the caller still holds it"
    );
    assert_eq!(pool.release_connection(id, 2_100), Ok(()));
    let metrics = pool.pool_metrics();
    assert_eq!(metrics.total_acquired, metrics.total_released);
}

#[test]
fn health_checks_mark_leased_expired_connection_draining() {
    let mut config = TlsPoolConfig::new();
    config.connection_lifetime_ms = 1_000;
    let mut pool = TlsConnectionPool::new(config);
    let id = pool.acquire_connection("tak.example.com:8089", 0).unwrap();
    assert_eq!(pool.run_health_checks(2_000).failed_count, 1);
    assert_eq!(pool.release_connection(id, 2_100), Ok(()));
}
