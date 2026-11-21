use xrtak_core::tls_pool::{TlsConnectionPool, TlsPoolConfig};

#[test]
fn leased_connection_of_a_removed_endpoint_is_closed_on_release() {
    let config = TlsPoolConfig::new()
        .with_endpoint("tak1.local:8089")
        .with_endpoint("tak2.local:8089");
    let mut pool = TlsConnectionPool::new(config);

    let id = pool.acquire_connection("tak1.local:8089", 0).unwrap();
    pool.remove_endpoint("tak1.local:8089");
    let _ = pool.release_connection(id, 10);

    assert_eq!(
        pool.pool_metrics().idle_connections,
        0,
        "connection to a removed endpoint went back to the idle set"
    );
    assert_ne!(
        pool.acquire_connection("tak1.local:8089", 20).ok(),
        Some(id),
        "connection of a removed endpoint was handed out again"
    );
}
