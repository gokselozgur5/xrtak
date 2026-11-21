use xrtak_core::tls_pool::{MutualCertAuthenticator, PeerCertificateInfo, TlsPoolConfig};

fn cert(cn: &str, sans: &[&str]) -> PeerCertificateInfo {
    PeerCertificateInfo {
        subject_cn: cn.into(),
        subject_alt_names: sans.iter().map(|s| s.to_string()).collect(),
        issuer: "Example CA".into(),
        not_before_epoch_ms: 0,
        not_after_epoch_ms: 10_000,
        sha256_fingerprint: "00".into(),
        is_ca: false,
    }
}

#[test]
fn peer_name_matching_follows_rfc_6125() {
    let config = TlsPoolConfig::new().with_client_identity("client", "AA");
    let check = |c: &PeerCertificateInfo, host: &str| {
        MutualCertAuthenticator::validate_peer(c, host, 5_000, &config)
    };

    // Wildcards apply to DNS names only; an IP address endpoint must match exactly.
    assert!(check(&cert("x", &["*.0.0.1"]), "10.0.0.1:8089").is_err());
    // The CN is ignored when subjectAltName entries are present (RFC 6125 6.4.4).
    assert!(check(&cert("tak.mil", &["other.example"]), "tak.mil:8089").is_err());
    // A bracketed IPv6 endpoint is compared by its address, not by "[".
    assert!(check(&cert("x", &["::1"]), "[::1]:8089").is_ok());
}
