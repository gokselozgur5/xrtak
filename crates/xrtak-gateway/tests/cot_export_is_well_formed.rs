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
fn exported_cot_escapes_markup_in_callsign_and_remarks() {
    let (gw, port) = start();
    let pin = r#"{"uid":"PIN-1","callsign":"R&D TEAM","cot_type":"a-f-G","east_m":1,"north_m":1}"#;
    send(port, "POST", "/api/spawn", pin);
    let mv = r#"{"uid":"PIN-1","east_m":2,"north_m":2,"remarks":"fuel < 20% & low"}"#;
    send(port, "POST", "/api/pin/move", mv);
    let xml = send(port, "GET", "/api/export/cot", "");
    stop(gw);
    assert!(xml.contains("PIN-1"), "pin missing from export");
    assert!(
        !xml.contains("R&D TEAM"),
        "unescaped ampersand in callsign attribute"
    );
    assert!(
        !xml.contains("fuel < 20% & low"),
        "unescaped markup in remarks text"
    );
}
