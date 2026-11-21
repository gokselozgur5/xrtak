use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

#[test]
fn idle_connections_do_not_terminate_the_process() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let addr = format!("127.0.0.1:{port}");
    let mut cmd = Command::new("sh");
    cmd.args([
        "-c",
        "ulimit -n 128; exec \"$0\"",
        env!("CARGO_BIN_EXE_xrtak-gateway"),
    ]);
    cmd.env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr);
    let mut gw = cmd.spawn().unwrap();
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let idle: Vec<_> = (0..200)
        .filter_map(|_| TcpStream::connect(&addr).ok())
        .collect();
    std::thread::sleep(Duration::from_millis(1000));
    let exited = gw.try_wait().unwrap();
    let _ = gw.kill();
    let _ = gw.wait();
    assert!(
        exited.is_none(),
        "gateway exited with {exited:?} after {} idle connections",
        idle.len()
    );
}

#[test]
fn stalled_content_length_body_times_out_and_releases_permits() {
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
        std::thread::sleep(Duration::from_millis(100));
    }
    let mut clients = Vec::new();
    for _ in 0..256 {
        if let Ok(mut s) = TcpStream::connect(&addr) {
            let _ = s.write_all(b"POST /api/terrain HTTP/1.1\r\nContent-Length: 100\r\n\r\n");
            clients.push(s);
        }
    }
    std::thread::sleep(Duration::from_millis(11000));
    let mut probe = TcpStream::connect(&addr).expect("must accept after permit release");
    let _ = probe.write_all(b"GET /api/state HTTP/1.1\r\n\r\n");
    let mut resp = [0u8; 12];
    let _ = probe.read(&mut resp);
    let _ = gw.kill();
    let _ = gw.wait();
    assert_eq!(&resp, b"HTTP/1.1 200");
}
