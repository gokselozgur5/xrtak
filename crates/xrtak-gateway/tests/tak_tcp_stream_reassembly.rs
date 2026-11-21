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
fn event_split_across_two_tcp_segments_is_ingested() {
    let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
    let up_addr = upstream.local_addr().unwrap();
    let (gw, port) = start();
    send(
        port,
        "POST",
        "/api/tak-config",
        &format!(r#"{{"tak_tcp_addr":"{up_addr}"}}"#),
    );
    let (mut tak, _) = upstream.accept().unwrap();
    tak.set_nodelay(true).unwrap();
    let ev =
        r#"<event uid="EV-SPLIT" type="a-f-G"><point lat="39.92" lon="32.85" hae="900"/></event>"#;
    tak.write_all(&ev.as_bytes()[..40]).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    tak.write_all(&ev.as_bytes()[40..]).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let state = send(port, "GET", "/api/state", "");
    stop(gw);
    assert!(
        state.contains("EV-SPLIT"),
        "event split across two reads was dropped"
    );
}
