use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::Duration;

fn start(dir: &std::path::Path) -> (Child, u16) {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
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
fn pins_created_with_spawn_survive_restart() {
    let dir = std::env::temp_dir().join(format!("xrtak-gw-spawn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let (gw1, port1) = start(&dir);
    let spawn_payload = r#"{"uid":"PIN-OP-TEST-1","callsign":"OP-RECON","cot_type":"a-f-G","east_m":100.0,"north_m":200.0}"#;
    let spawn_res = send(port1, "POST", "/api/spawn", spawn_payload);
    assert!(
        spawn_res.starts_with("HTTP/1.1 200"),
        "spawn failed: {spawn_res}"
    );
    stop(gw1);

    // Verify pin was written to real_pins.json
    let store = dir.join("data/real_pins.json");
    let pins = std::fs::read_to_string(&store).unwrap_or_default();
    assert!(pins.contains("PIN-OP-TEST-1"), "pin missing from store");

    // Start second gateway instance on same directory and verify it loads the pin
    let (gw2, port2) = start(&dir);
    let state = send(port2, "GET", "/api/state", "");
    stop(gw2);
    assert!(
        state.contains("PIN-OP-TEST-1"),
        "pin did not reload in second instance"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
