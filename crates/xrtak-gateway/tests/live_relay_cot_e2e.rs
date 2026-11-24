use std::net::{TcpListener, UdpSocket};
use std::process::{Child, Command};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

fn start_gateway_ephemeral() -> (Child, u16, u16) {
    let http_port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let udp_port = UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    let http_addr = format!("127.0.0.1:{http_port}");
    let udp_addr = format!("127.0.0.1:{udp_port}");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.env("XRTAK_HTTP_ADDR", &http_addr)
        .env("XRTAK_UDP_ADDR", &udp_addr);

    let child = cmd.spawn().expect("failed to spawn xrtak-gateway");

    for _ in 0..50 {
        if std::net::TcpStream::connect(&http_addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(150));
    (child, http_port, udp_port)
}

fn stop_child(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_live_relay_cot_udp_ingestion_and_world_query() {
    let (child, http_port, udp_port) = start_gateway_ephemeral();

    // 1. Send airborne UAV telemetry CoT (type="a-u-A-M-F-Q")
    let uav_cot = r#"<event version="2.0" uid="UAV-RELAY-01" type="a-u-A-M-F-Q" how="m-g">
<point lat="39.9250" lon="32.8360" hae="1150.0" ce="10" le="10"/>
<detail>
  <contact callsign="SKY-GUARDIAN-1"/>
  <track course="85.5" speed="26.0"/>
  <remarks>Airborne CoT Relay UAV Telemetry</remarks>
</detail>
</event>"#;

    // 2. Send ground vehicle detection CoT (type="a-u-G-E-V-C")
    let gnd_cot = r#"<event version="2.0" uid="GND-RELAY-TARGET-02" type="a-u-G-E-V-C" how="m-g">
<point lat="39.9180" lon="32.8450" hae="920.0" ce="5" le="5"/>
<detail>
  <contact callsign="CONVOY-LEAD"/>
  <track course="180.0" speed="12.5"/>
  <remarks>Ground Vehicle Track</remarks>
</detail>
</event>"#;

    let client_sock = UdpSocket::bind("127.0.0.1:0").expect("bind client udp");
    let target_addr = format!("127.0.0.1:{udp_port}");

    client_sock
        .send_to(uav_cot.as_bytes(), &target_addr)
        .expect("send uav cot");
    client_sock
        .send_to(gnd_cot.as_bytes(), &target_addr)
        .expect("send gnd cot");

    // Allow gateway async ingest loop to process UDP packet
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Query GET /api/world
    let mut tcp = TcpStream::connect(("127.0.0.1", http_port))
        .await
        .expect("connect to http");
    use tokio::io::AsyncWriteExt;
    tcp.write_all(b"GET /api/world HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .expect("send get req");

    let mut body = Vec::new();
    tcp.read_to_end(&mut body).await.expect("read response");

    stop_child(child);

    let resp_str = String::from_utf8_lossy(&body);
    assert!(
        resp_str.starts_with("HTTP/1.1 200 OK"),
        "unexpected response: {resp_str}"
    );

    // Verify UAV track
    assert!(
        resp_str.contains("UAV-RELAY-01"),
        "missing UAV-RELAY-01 in world JSON"
    );
    assert!(
        resp_str.contains("SKY-GUARDIAN-1"),
        "missing SKY-GUARDIAN-1 callsign in world JSON"
    );
    assert!(
        resp_str.contains("a-u-A-M-F-Q"),
        "missing a-u-A-M-F-Q type in world JSON"
    );

    // Verify Ground vehicle track
    assert!(
        resp_str.contains("GND-RELAY-TARGET-02"),
        "missing GND-RELAY-TARGET-02 in world JSON"
    );
    assert!(
        resp_str.contains("CONVOY-LEAD"),
        "missing CONVOY-LEAD callsign in world JSON"
    );
    assert!(
        resp_str.contains("a-u-G-E-V-C"),
        "missing a-u-G-E-V-C type in world JSON"
    );
}
