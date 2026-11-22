use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::time::Duration;

fn start_gateway() -> (Child, u16) {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let addr = format!("127.0.0.1:{port}");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr);
    let child = cmd.spawn().unwrap();
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(150));
    (child, port)
}

fn stop_gateway(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn send_raw(port: u16, req: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(req.as_bytes()).unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    out
}

fn send_req(port: u16, method: &str, path: &str, body: &str) -> String {
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    send_raw(port, &req)
}

#[test]
fn test_request_parser_complete_and_malformed() {
    let (gw, port) = start_gateway();
    let resp = send_req(port, "GET", "/api/state", "");
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "expected 200, got: {resp}");

    let malformed = send_raw(port, "GARBAGE\r\n\r\n");
    assert!(malformed.is_empty() || malformed.starts_with("HTTP/1.1 404"), "expected empty or 404, got: {malformed}");

    stop_gateway(gw);
}
