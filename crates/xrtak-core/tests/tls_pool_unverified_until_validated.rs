use xrtak_core::tls_pool::{TlsConnectionPool, TlsPoolConfig};

#[test]
fn new_connection_is_not_reported_as_verified_before_any_peer_validation() {
    let config = TlsPoolConfig::new().with_client_identity("client-xr", "A1B2C3D4");
    let mut pool = TlsConnectionPool::new(config);
    let id = pool.acquire_connection("203.0.113.9:8089", 1_000).unwrap();
    let conn = pool.get_connection(id).unwrap();
    assert!(
        !conn.is_mtls_verified,
        "connection reported as mTLS verified without a peer certificate"
    );
    assert_ne!(
        conn.peer_identity.as_deref(),
        Some("client-xr"),
        "peer identity is a copy of the local client certificate CN"
    );
}
