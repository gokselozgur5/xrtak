use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::time::Duration;

fn start_gateway() -> (Child, u16) {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let addr = format!("127.0.0.1:{port}");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.env("XRTAK_HTTP_ADDR", &addr)
        .env("XRTAK_UDP_ADDR", &addr);
    let child = cmd.spawn().unwrap();
    for _ in 0..50 {
        if TcpStream::connect(&addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(150));
    (child, port)
}

fn stop_gateway(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn send_raw(port: u16, req: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(req.as_bytes()).unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    out
}

fn send_req(port: u16, method: &str, path: &str, body: &str) -> String {
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    send_raw(port, &req)
}

#[test]
fn test_request_parser_complete_and_malformed() {
    let (gw, port) = start_gateway();
    let resp = send_req(port, "GET", "/api/state", "");
    assert!(resp.starts_with("HTTP/1.1 200 OK"), "expected 200, got: {resp}");

    let malformed = send_raw(port, "GARBAGE\r\n\r\n");
    assert!(malformed.is_empty() || malformed.starts_with("HTTP/1.1 404"), "expected empty or 404, got: {malformed}");

    stop_gateway(gw);
}

#[test]
fn test_request_parser_split_body() {
    let (gw, port) = start_gateway();
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let head = "POST /api/viewshed HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 24\r\nConnection: close\r\n\r\n";
    s.write_all(head.as_bytes()).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let part1 = b"{\"east_m\":0,";
    s.write_all(part1).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let part2 = b" \"north_m\":0}";
    s.write_all(part2).unwrap();

    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    stop_gateway(gw);
    assert!(out.starts_with("HTTP/1.1 200 OK"), "got: {out}");
}

#[test]
fn test_core_api_routes_status_and_shape() {
    let (gw, port) = start_gateway();

    // 1. GET /
    let res = send_req(port, "GET", "/", "");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "GET /: {res}");

    // 2. OPTIONS /api/state
    let res = send_req(port, "OPTIONS", "/api/state", "");
    assert!(res.starts_with("HTTP/1.1 204 No Content"), "OPTIONS: {res}");

    // 3. GET /api/state
    let res = send_req(port, "GET", "/api/state", "");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "GET /api/state: {res}");
    assert!(res.contains("\"origin\""), "shape /api/state");

    // 4. GET /api/aar
    let res = send_req(port, "GET", "/api/aar", "");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "GET /api/aar: {res}");
    assert!(res.contains("\"summary\""), "shape /api/aar");

    // 5. GET /api/hud
    let res = send_req(port, "GET", "/api/hud", "");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "GET /api/hud: {res}");
    assert!(res.contains("\"cues\""), "shape /api/hud");

    // 6. GET /api/terrain
    let res = send_req(port, "GET", "/api/terrain", "");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "GET /api/terrain: {res}");
    assert!(res.contains("\"heights_m\""), "shape /api/terrain");

    // 7. POST /api/terrain
    let res = send_req(port, "POST", "/api/terrain", "{\"preset\":\"flat\",\"extent_m\":1000.0}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/terrain: {res}");

    // 8. POST /api/terrain/geotiff
    let res = send_req(port, "POST", "/api/terrain/geotiff", "{\"name\":\"test\",\"bytes\":[]}");
    assert!(res.starts_with("HTTP/1.1 400 Bad Request") || res.starts_with("HTTP/1.1 200 OK"), "POST /api/terrain/geotiff: {res}");

    // 9. POST /api/cot
    let cot_body = "<event version=\"2.0\" uid=\"T1\" type=\"a-f-G\" time=\"2026-01-01T00:00:00Z\" start=\"2026-01-01T00:00:00Z\" stale=\"2026-01-01T01:00:00Z\" how=\"m-g\"><point lat=\"10.0\" lon=\"20.0\" hae=\"0.0\" ce=\"1.0\" le=\"1.0\"/></event>";
    let res = send_req(port, "POST", "/api/cot", cot_body);
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/cot: {res}");

    stop_gateway(gw);
}

#[test]
fn test_mission_and_analysis_routes() {
    let (gw, port) = start_gateway();

    // 10. POST /api/spi
    let res = send_req(port, "POST", "/api/spi", "{\"sensor_uid\":\"U1\",\"clear\":true}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/spi: {res}");

    // 11. POST /api/viewshed
    let res = send_req(port, "POST", "/api/viewshed", "{\"east_m\":0.0,\"north_m\":0.0}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/viewshed: {res}");
    assert!(res.contains("\"visible_ratio\""), "shape /api/viewshed");

    // 12. POST /api/los
    let res = send_req(port, "POST", "/api/los", "{\"from_enu\":[0.0,10.0,0.0],\"to_enu\":[100.0,10.0,100.0]}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/los: {res}");
    assert!(res.contains("\"clear\""), "shape /api/los");

    // 13. POST /api/elevation-profile
    let res = send_req(port, "POST", "/api/elevation-profile", "{\"waypoints_en\":[[0.0,0.0],[100.0,100.0]]}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/elevation-profile: {res}");
    assert!(res.contains("\"samples\""), "shape /api/elevation-profile");

    // 14. POST /api/mission/import
    let res = send_req(port, "POST", "/api/mission/import", "{\"content\":\"\"}");
    assert!(res.starts_with("HTTP/1.1 400 Bad Request") || res.starts_with("HTTP/1.1 200 OK"), "POST /api/mission/import: {res}");

    // 15. POST /api/mission/export
    let res = send_req(port, "POST", "/api/mission/export", "{\"waypoints_en_agl\":[[0.0,0.0,50.0]]}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/mission/export: {res}");

    stop_gateway(gw);
}
