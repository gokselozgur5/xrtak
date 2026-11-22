use xrtak_core::tls_pool::{TlsConnectionPool, TlsPoolConfig};

#[test]
fn prune_keeps_min_idle_when_another_connection_expires() {
    let mut config = TlsPoolConfig::new();
    config.min_idle_connections = 1;
    config.idle_timeout_ms = 500;
    config.connection_lifetime_ms = 2_000;
    let mut pool = TlsConnectionPool::new(config);

    let old = pool.acquire_connection("tak:8089", 0).unwrap();
    let young = pool.acquire_connection("tak:8089", 1_000).unwrap();
    pool.release_connection(old, 1_100).unwrap();
    pool.release_connection(young, 1_100).unwrap();
    // At t = 2100 "old" is past its lifetime and "young" is only idle-timed-out.
    pool.prune_idle_and_expired(2_100);
    assert_eq!(
        pool.pool_metrics().idle_connections,
        1,
        "min_idle_connections = 1"
    );
}
