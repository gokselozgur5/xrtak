use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

struct GatewayGuard(Child);
impl Drop for GatewayGuard {
    fn drop(&mut self) {
        let _ = (self.0.kill(), self.0.wait());
    }
}

fn post_head(port: u16, body: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let hdr = format!(
        "POST /api/elevation-profile HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = (s.write_all(hdr.as_bytes()), s.write_all(body.as_bytes()));
    let mut buf = [0u8; 256];
    let n = s.read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

#[test]
fn elevation_profile_rejects_waypoints_over_cap_and_stays_responsive() {
    let sock = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = format!("127.0.0.1:{}", sock.local_addr().unwrap().port());
    let port = sock.local_addr().unwrap().port();
    drop(sock);
    let _gw = GatewayGuard(
        Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"))
            .env("XRTAK_HTTP_ADDR", &addr)
            .env("XRTAK_UDP_ADDR", &addr)
            .spawn()
            .unwrap(),
    );
    while TcpStream::connect(&addr).is_err() {
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(100));
    let body_n = |n| format!("{{\"waypoints_en\":[{}]}}", vec!["[1.0,2.0]"; n].join(","));
    assert!(post_head(port, &body_n(64)).starts_with("HTTP/1.1 200"));
    assert!(post_head(port, &body_n(65)).starts_with("HTTP/1.1 400"));
    let (huge, t0) = (body_n(100_000), Instant::now());
    assert!(post_head(port, &huge).starts_with("HTTP/1.1 400"));
    assert!(t0.elapsed() < Duration::from_secs(2));
    assert!(post_head(port, &body_n(2)).starts_with("HTTP/1.1 200"));
}
