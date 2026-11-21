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
fn headers_split_across_two_writes_are_still_parsed() {
    let (gw, port) = start();
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.write_all(b"POST /api/viewshed HTTP/1.1\r\nHost: x\r\n")
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let _ = s.write_all(b"Content-Length: 24\r\n\r\n{\"east_m\":0, \"north_m\":0}");
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    stop(gw);
    assert!(
        out.starts_with("HTTP/1.1 200"),
        "got: {:?}",
        out.lines().next()
    );
}

#[test]
fn body_over_four_mebibytes_gets_an_http_response() {
    let (gw, port) = start();
    let body = format!(r#"{{"bytes":[{}0]}}"#, "0,".repeat(2_600_000));
    let out = send(port, "POST", "/api/terrain/geotiff", &body);
    stop(gw);
    let handled = out.starts_with("HTTP/1.1 413") || out.contains("\"error\"");
    assert!(
        handled,
        "expected 413 or a parsed-body error, got: {:?}",
        out.lines().next()
    );
}
