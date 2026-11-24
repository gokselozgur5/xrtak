use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::Duration;

fn start() -> (Child, u16) {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let dir = std::env::temp_dir().join(format!("xrtak-gw-{port}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.current_dir(dir)
        .env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr);
    let child = cmd.spawn().unwrap();
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(200));
    (child, port)
}

fn stop(mut gw: Child) {
    let _ = gw.kill();
    let _ = gw.wait();
}

fn send(port: u16, method: &str, path: &str, body: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let _ = write!(
        s,
        "{method} {path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    out
}

#[test]
fn cross_origin_post_cannot_reconfigure_the_gateway() {
    let (gw, port) = start();
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let body = r#"{"sim_enabled":false,"tak_tcp_addr":"127.0.0.1:9"}"#;
    let len = body.len();
    write!(s, "POST /api/tak-config HTTP/1.1\r\nOrigin: http://evil.example\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}").unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    let state = send(port, "GET", "/api/state", "");
    stop(gw);
    assert!(
        !out.starts_with("HTTP/1.1 200"),
        "unauthenticated cross-origin POST was accepted"
    );
    assert!(
        state.contains(r#""sim_enabled":true"#),
        "state was changed by the cross-origin POST"
    );
}

#[test]
fn origin_with_mixed_case_and_no_space_is_rejected() {
    let (gw, port) = start();
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let body = r#"{"sim_enabled":false,"tak_tcp_addr":"127.0.0.1:9"}"#;
    let len = body.len();
    write!(
        s,
        "POST /api/tak-config HTTP/1.1\r\nORIGIN:http://evil.example\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}"
    )
    .unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    let state = send(port, "GET", "/api/state", "");
    stop(gw);
    assert!(!out.starts_with("HTTP/1.1 200"));
    assert!(state.contains(r#""sim_enabled":true"#));
}

#[test]
fn token_auth_rejects_missing_and_accepts_canonicalized() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let dir = std::env::temp_dir().join(format!("xrtak-gw-tok-{port}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let addr = format!("127.0.0.1:{port}");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.current_dir(dir)
        .env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr)
        .env("XRTAK_TOKEN", "secret-token-123");
    let mut gw = cmd.spawn().unwrap();
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(200));
    let body = r#"{"sim_enabled":false}"#;
    let len = body.len();
    let mut s1 = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(s1, "POST /api/tak-config HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {len}\r\n\r\n{body}").unwrap();
    let mut out1 = String::new();
    let _ = s1.read_to_string(&mut out1);
    assert!(out1.starts_with("HTTP/1.1 401 Unauthorized"));
    let mut s2 = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(s2, "POST /api/tak-config HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Xrtak-Token: secret-token-123\r\nContent-Length: {len}\r\n\r\n{body}").unwrap();
    let mut out2 = String::new();
    let _ = s2.read_to_string(&mut out2);
    assert!(out2.starts_with("HTTP/1.1 200 OK"));
    let _ = gw.kill();
    let _ = gw.wait();
}

#[test]
fn tak_tcp_addr_reconfiguration_replaces_prior_connection() {
    let mock1 = TcpListener::bind("127.0.0.1:0").unwrap();
    let port1 = mock1.local_addr().unwrap().port();
    let mock2 = TcpListener::bind("127.0.0.1:0").unwrap();
    let port2 = mock2.local_addr().unwrap().port();
    let (gw, port) = start();
    let b1 = format!(r#"{{"tak_tcp_addr":"127.0.0.1:{port1}"}}"#);
    let _ = send(port, "POST", "/api/tak-config", &b1);
    let (mut client1, _) = mock1.accept().unwrap();
    let b2 = format!(r#"{{"tak_tcp_addr":"127.0.0.1:{port2}"}}"#);
    let _ = send(port, "POST", "/api/tak-config", &b2);
    let (_client2, _) = mock2.accept().unwrap();
    let mut buf = [0u8; 16];
    client1
        .set_read_timeout(Some(Duration::from_millis(1500)))
        .unwrap();
    loop {
        match client1.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    stop(gw);
}
