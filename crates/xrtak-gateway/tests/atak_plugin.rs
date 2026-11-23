use std::net::{TcpListener, UdpSocket};
use std::process::{Child, Command};
use std::time::Duration;

fn start_with_feedback(feedback_addr: &str) -> (Child, u16) {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let addr = format!("127.0.0.1:{port}");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr)
        .env("XRTAK_FEEDBACK_ADDR", feedback_addr);
    let child = cmd.spawn().unwrap();
    for _ in 0..50 {
        if std::net::TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(150));
    (child, port)
}

fn stop(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn feedback_events_reach_configured_address() {
    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap().to_string();
    receiver
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();

    let (gw, port) = start_with_feedback(&recv_addr);

    let pin = r#"{"uid":"ATAK-PIN-1","callsign":"BRIDGE-TEST","cot_type":"a-f-G","east_m":10,"north_m":20}"#;
    let req = format!(
        "POST /api/spawn HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{pin}",
        pin.len()
    );
    use std::io::Write;
    let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.write_all(req.as_bytes()).unwrap();

    let mut buf = [0u8; 2048];
    let n = receiver.recv(&mut buf).unwrap();
    stop(gw);

    let msg = String::from_utf8_lossy(&buf[..n]);
    assert!(
        msg.contains("ATAK-PIN-1"),
        "expected feedback XML to contain ATAK-PIN-1"
    );
}
