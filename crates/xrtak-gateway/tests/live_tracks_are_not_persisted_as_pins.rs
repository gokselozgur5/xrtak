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
fn udp_track_is_not_written_to_the_pin_store() {
    let (gw, port) = start();
    let ev = r#"<event uid="LIVE-UDP-1" type="a-f-A"><point lat="39.921" lon="32.851" hae="1200"/></event>"#;
    let sock = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sock.send_to(ev.as_bytes(), ("127.0.0.1", port)).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let state = send(port, "GET", "/api/state", "");
    send(port, "POST", "/api/tak-config", r#"{"sim_enabled":true}"#);
    stop(gw);
    assert!(state.contains("LIVE-UDP-1"), "datagram was not ingested");
    let store = std::env::temp_dir().join(format!("xrtak-gw-{port}/data/real_pins.json"));
    let pins = std::fs::read_to_string(store).unwrap_or_default();
    assert!(
        !pins.contains("LIVE-UDP-1"),
        "live UDP track was saved as a persistent pin"
    );
}
