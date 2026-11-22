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
    assert!(
        resp.starts_with("HTTP/1.1 200 OK"),
        "expected 200, got: {resp}"
    );

    let malformed = send_raw(port, "GARBAGE\r\n\r\n");
    assert!(
        malformed.is_empty() || malformed.starts_with("HTTP/1.1 404"),
        "expected empty or 404, got: {malformed}"
    );

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
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "GET /api/terrain: {res}"
    );
    assert!(res.contains("\"heights_m\""), "shape /api/terrain");

    // 7. POST /api/terrain
    let res = send_req(
        port,
        "POST",
        "/api/terrain",
        "{\"preset\":\"flat\",\"extent_m\":1000.0}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/terrain: {res}"
    );

    // 8. POST /api/terrain/geotiff
    let res = send_req(
        port,
        "POST",
        "/api/terrain/geotiff",
        "{\"name\":\"test\",\"bytes\":[]}",
    );
    assert!(
        res.starts_with("HTTP/1.1 400 Bad Request") || res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/terrain/geotiff: {res}"
    );

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
    let res = send_req(
        port,
        "POST",
        "/api/spi",
        "{\"sensor_uid\":\"U1\",\"clear\":true}",
    );
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/spi: {res}");

    // 11. POST /api/viewshed
    let res = send_req(
        port,
        "POST",
        "/api/viewshed",
        "{\"east_m\":0.0,\"north_m\":0.0}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/viewshed: {res}"
    );
    assert!(res.contains("\"visible_ratio\""), "shape /api/viewshed");

    // 12. POST /api/los
    let res = send_req(
        port,
        "POST",
        "/api/los",
        "{\"from_enu\":[0.0,10.0,0.0],\"to_enu\":[100.0,10.0,100.0]}",
    );
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/los: {res}");
    assert!(res.contains("\"clear\""), "shape /api/los");

    // 13. POST /api/elevation-profile
    let res = send_req(
        port,
        "POST",
        "/api/elevation-profile",
        "{\"waypoints_en\":[[0.0,0.0],[100.0,100.0]]}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/elevation-profile: {res}"
    );
    assert!(res.contains("\"samples\""), "shape /api/elevation-profile");

    // 14. POST /api/mission/import
    let res = send_req(port, "POST", "/api/mission/import", "{\"content\":\"\"}");
    assert!(
        res.starts_with("HTTP/1.1 400 Bad Request") || res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/mission/import: {res}"
    );

    // 15. POST /api/mission/export
    let res = send_req(
        port,
        "POST",
        "/api/mission/export",
        "{\"waypoints_en_agl\":[[0.0,0.0,50.0]]}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/mission/export: {res}"
    );

    stop_gateway(gw);
}

#[test]
fn test_spatial_and_tactical_planning_routes() {
    let (gw, port) = start_gateway();

    // 16. POST /api/tiles/resolve
    let res = send_req(
        port,
        "POST",
        "/api/tiles/resolve",
        "{\"lat\":37.0,\"lon\":-122.0,\"zoom\":10}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/tiles/resolve: {res}"
    );

    // 17. POST /api/mission/corridor-polygon
    let res = send_req(
        port,
        "POST",
        "/api/mission/corridor-polygon",
        "{\"waypoints_en\":[[0.0,0.0],[100.0,100.0]]}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/mission/corridor-polygon: {res}"
    );

    // 18. POST /api/mapset/plan
    let res = send_req(
        port,
        "POST",
        "/api/mapset/plan",
        "{\"south\":36.0,\"west\":-122.0,\"north\":37.0,\"east\":-121.0}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/mapset/plan: {res}"
    );

    // 19. POST /api/rf/link-budget
    let res = send_req(
        port,
        "POST",
        "/api/rf/link-budget",
        "{\"tx_enu\":[0.0,10.0,0.0],\"rx_enu\":[500.0,10.0,500.0]}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/rf/link-budget: {res}"
    );

    // 20. POST /api/tracks/predict
    let res = send_req(port, "POST", "/api/tracks/predict", "{\"uid\":\"P1\",\"pos_enu\":[0.0,0.0,0.0],\"vel_enu\":[1.0,0.0,0.0],\"dt_sec\":1.0,\"steps\":5}");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/tracks/predict: {res}"
    );

    // 21. POST /api/airspace/deconflict
    let res = send_req(port, "POST", "/api/airspace/deconflict", "{\"tracks\":[]}");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/airspace/deconflict: {res}"
    );

    // 22. GET /api/export/opord
    let res = send_req(port, "GET", "/api/export/opord", "");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "GET /api/export/opord: {res}"
    );
    assert!(
        res.contains("\"opord_markdown\""),
        "shape /api/export/opord"
    );

    stop_gateway(gw);
}

#[test]
fn test_advanced_domain_analysis_routes() {
    let (gw, port) = start_gateway();

    // 23. POST /api/ew/jamming
    let ew_body = serde_json::json!({
        "jammer": {
            "callsign": "JAM",
            "transmitter_power_w": 10.0,
            "antenna_gain_dbi": 10.0,
            "bandwidth_mhz": 20.0,
            "east_m": 0.0,
            "north_m": 100.0,
            "up_m": 50.0
        },
        "link": {
            "callsign": "LINK",
            "transmitter_power_w": 1.0,
            "tx_antenna_gain_dbi": 2.0,
            "rx_antenna_gain_dbi": 2.0,
            "signal_bandwidth_mhz": 10.0,
            "receiver_sensitivity_dbm": -95.0,
            "tx_east_m": 0.0,
            "tx_north_m": 5000.0,
            "tx_up_m": 100.0,
            "rx_east_m": 0.0,
            "rx_north_m": 0.0,
            "rx_up_m": 10.0
        }
    })
    .to_string();
    let res = send_req(port, "POST", "/api/ew/jamming", &ew_body);
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/ew/jamming: {res}"
    );
    assert!(res.contains("\"assessment\""), "shape /api/ew/jamming");

    // 24. POST /api/medevac/nine-line
    let med_body = serde_json::json!({
        "location": "GRID 123 456",
        "freq_callsign": "30.00 DUSTOFF",
        "enu_center": [0.0, 0.0, 0.0]
    })
    .to_string();
    let res = send_req(port, "POST", "/api/medevac/nine-line", &med_body);
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/medevac/nine-line: {res}"
    );
    assert!(
        res.contains("\"formatted_nine_line\""),
        "shape /api/medevac/nine-line"
    );

    // 25. POST /api/fires/ballistics
    let res = send_req(
        port,
        "POST",
        "/api/fires/ballistics",
        "{\"origin_enu\":[0.0,0.0,0.0],\"target_enu\":[1000.0,0.0,1000.0]}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/fires/ballistics: {res}"
    );
    assert!(res.contains("\"solution\""), "shape /api/fires/ballistics");

    // 26. POST /api/uav/endurance
    let uav_body = serde_json::json!({
        "state": {
            "callsign": "VALK",
            "battery_remaining_pct": 50.0,
            "battery_capacity_wh": 100.0,
            "current_power_draw_w": 200.0,
            "reserve_margin_pct": 15.0,
            "cruise_airspeed_mps": 15.0,
            "current_east_m": 0.0,
            "current_north_m": 0.0,
            "current_up_m": 500.0,
            "home_east_m": 0.0,
            "home_north_m": 0.0,
            "home_up_m": 0.0
        }
    })
    .to_string();
    let res = send_req(port, "POST", "/api/uav/endurance", &uav_body);
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/uav/endurance: {res}"
    );
    assert!(res.contains("\"solution\""), "shape /api/uav/endurance");

    stop_gateway(gw);
}

#[test]
fn test_pin_management_and_export_routes() {
    let (gw, port) = start_gateway();

    // 27. POST /api/spawn
    let res = send_req(port, "POST", "/api/spawn", "{\"uid\":\"PIN-MOVE-1\",\"callsign\":\"TEST-PIN\",\"cot_type\":\"a-f-G\",\"east_m\":10.0,\"north_m\":20.0}");
    assert!(res.starts_with("HTTP/1.1 200 OK"), "POST /api/spawn: {res}");

    // 28. POST /api/pin/move
    let res = send_req(
        port,
        "POST",
        "/api/pin/move",
        "{\"uid\":\"PIN-MOVE-1\",\"east_m\":15.0,\"north_m\":25.0}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/pin/move: {res}"
    );

    // 29. POST /api/pin/delete
    let res = send_req(port, "POST", "/api/pin/delete", "{\"uid\":\"PIN-MOVE-1\"}");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/pin/delete: {res}"
    );

    // 30. GET /api/export/geojson
    let res = send_req(port, "GET", "/api/export/geojson", "");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "GET /api/export/geojson: {res}"
    );
    assert!(
        res.contains("\"FeatureCollection\""),
        "shape /api/export/geojson"
    );

    // 31. GET /api/export/cot
    let res = send_req(port, "GET", "/api/export/cot", "");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "GET /api/export/cot: {res}"
    );
    assert!(res.contains("<events"), "shape /api/export/cot");

    stop_gateway(gw);
}

#[test]
fn test_collab_sapient_and_tak_config_routes() {
    let (gw, port) = start_gateway();

    // 32. POST /api/pointer
    let res = send_req(
        port,
        "POST",
        "/api/pointer",
        "{\"east_m\":0.0,\"north_m\":0.0}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/pointer: {res}"
    );

    // 33. POST /api/collab/sync
    let res = send_req(
        port,
        "POST",
        "/api/collab/sync",
        "{\"operator_id\":\"OP-1\",\"callsign\":\"ALFA\"}",
    );
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/collab/sync: {res}"
    );

    // 34. POST /api/sapient
    let sap_body = serde_json::json!({
        "node_id": "RADAR-1",
        "object_id": "T90-A",
        "classification": "TANK",
        "confidence": 0.95,
        "east_m": 100.0,
        "north_m": 200.0
    })
    .to_string();
    let res = send_req(port, "POST", "/api/sapient", &sap_body);
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/sapient: {res}"
    );

    // 35. POST /api/tak-config
    let res = send_req(port, "POST", "/api/tak-config", "{\"sim_enabled\":true}");
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/tak-config: {res}"
    );

    stop_gateway(gw);
}

#[test]
fn test_medevac_nine_line_enu_coordinates_order() {
    let (gw, port) = start_gateway();

    let med_body = serde_json::json!({
        "location": "LZ HAWK",
        "freq_callsign": "30.00 DUSTOFF",
        "enu_center": [250.0, 45.0, 350.0]
    })
    .to_string();
    let res = send_req(port, "POST", "/api/medevac/nine-line", &med_body);
    assert!(
        res.starts_with("HTTP/1.1 200 OK"),
        "POST /api/medevac/nine-line: {res}"
    );

    let body = res.split_once("\r\n\r\n").map(|x| x.1).unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(body).expect("valid json response");

    let lat = json["report"]["lat"].as_f64().expect("lat field");
    let lon = json["report"]["lon"].as_f64().expect("lon field");

    assert!(
        (lat - 39.923152).abs() < 1e-4,
        "lat should match north offset 350m: got {lat}"
    );
    assert!(
        (lon - 32.852926).abs() < 1e-4,
        "lon should match east offset 250m: got {lon}"
    );

    stop_gateway(gw);
}

#[test]
fn test_collab_sync_peer_map_capacity_cap() {
    let (gw, port) = start_gateway();

    for i in 0..150 {
        let body = serde_json::json!({
            "operator_id": format!("OP-{i}"),
            "callsign": format!("PEER-{i}")
        })
        .to_string();
        let _ = send_req(port, "POST", "/api/collab/sync", &body);
    }

    let check_body = serde_json::json!({
        "operator_id": "OP-PROBE",
        "callsign": "PROBE"
    })
    .to_string();
    let res = send_req(port, "POST", "/api/collab/sync", &check_body);
    assert!(res.starts_with("HTTP/1.1 200 OK"), "expected 200 OK: {res}");

    let body = res.split_once("\r\n\r\n").map(|x| x.1).unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(body).expect("valid json response");
    let ops = json["operators"].as_array().expect("operators array");
    assert!(
        ops.len() <= 64,
        "operator peer map exceeded capacity cap: got {}",
        ops.len()
    );

    stop_gateway(gw);
}

#[test]
fn test_gateway_fallback_retains_configured_host() {
    let dummy_listener = std::net::TcpListener::bind("[::1]:0").unwrap();
    let occupied_port = dummy_listener.local_addr().unwrap().port();

    let configured_addr = format!("[::1]:{occupied_port}");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xrtak-gateway"));
    cmd.env("XRTAK_HTTP_ADDR", &configured_addr)
        .env("XRTAK_UDP_ADDR", "127.0.0.1:0");
    let mut child = cmd.spawn().unwrap();

    let fallback_addr = "[::1]:8091";
    let mut connected = false;
    for _ in 0..30 {
        if TcpStream::connect(fallback_addr).is_ok() {
            connected = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = child.kill();
    let _ = child.wait();
    drop(dummy_listener);

    assert!(
        connected,
        "gateway did not fall back to port 8091 on configured host [::1]"
    );
}
