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
