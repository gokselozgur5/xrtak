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
