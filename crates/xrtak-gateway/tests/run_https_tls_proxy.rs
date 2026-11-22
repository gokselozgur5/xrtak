use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

#[test]
fn idle_client_does_not_block_second_client() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let root = env!("CARGO_MANIFEST_DIR");
    let script = format!("{root}/../../scripts/run_https_tls_proxy.py");
    let mut child = Command::new("python3")
        .args([&script])
        .env("XRTAK_HTTPS_PORT", port.to_string())
        .env("XRTAK_UPSTREAM_PORT", "9")
        .spawn()
        .unwrap();
    let addr = format!("127.0.0.1:{port}");
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let py = "import socket, ssl, sys\np=int(sys.argv[1])\na=('127.0.0.1', p)\ns1=socket.socket(); s1.connect(a)\nctx=ssl.create_default_context(); ctx.check_hostname=False; ctx.verify_mode=ssl.CERT_NONE\ns2=socket.socket(); s2.settimeout(2.0); s2=ctx.wrap_socket(s2); s2.connect(a)\ns1.close(); s2.close()\n";
    let res = Command::new("python3")
        .args(["-c", py, &port.to_string()])
        .output()
        .unwrap();
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        res.status.success(),
        "second client blocked: {}",
        String::from_utf8_lossy(&res.stderr)
    );
}
