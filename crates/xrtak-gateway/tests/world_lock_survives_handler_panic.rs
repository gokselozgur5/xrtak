use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

#[test]
fn terrain_upload_with_overflowing_dimensions_leaves_api_usable() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let addr = format!("127.0.0.1:{port}");
    let mut gw = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"))
        .env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr)
        .spawn()
        .unwrap();
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let send = |m, p, b: &str| {
        let mut s = TcpStream::connect(&addr).unwrap();
        let _ = write!(
            s,
            "{m} {p} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{b}",
            b.len()
        );
        let mut out = String::new();
        let _ = s.read_to_string(&mut out);
        out
    };
    send(
        "POST",
        "/api/terrain",
        r#"{"width":9223372036854775808,"height":2,"heights_m":[]}"#,
    );
    let after = send("GET", "/api/state", "");
    let _ = gw.kill();
    let _ = gw.wait();
    assert!(after.starts_with("HTTP/1.1 200"), "API dead: {after:?}");
}
