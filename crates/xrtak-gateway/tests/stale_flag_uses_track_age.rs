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
fn stale_flag_follows_age_not_speed() {
    let (gw, port) = start();
    let ev = |uid: &str, speed: &str| {
        format!(
            r#"<event uid="{uid}" type="a-f-G"><point lat="39.92" lon="32.85" hae="900"/><detail><track course="0" speed="{speed}"/></detail></event>"#
        )
    };
    send(port, "POST", "/api/cot", &ev("OLD-FAST", "40"));
    std::thread::sleep(Duration::from_secs(3));
    send(port, "POST", "/api/cot", &ev("NEW-SLOW", "1"));
    let req = r#"{"horizons_sec":[10],"stale_threshold_sec":2}"#;
    let out = send(port, "POST", "/api/tracks/predict", req);
    stop(gw);
    let stale = |uid: &str| {
        let seg = out.split(&format!(r#""uid":"{uid}""#)).nth(1).unwrap();
        seg[..seg.find("\"points\"").unwrap()].contains(r#""is_stale":true"#)
    };
    assert!(
        stale("OLD-FAST"),
        "a track last updated 3 s ago is stale at a 2 s threshold"
    );
    assert!(!stale("NEW-SLOW"), "a track updated just now is not stale");
}
