use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[test]
fn demo_probe_detects_running_gateway_under_pipefail() {
    let listener = TcpListener::bind("127.0.0.1:8090").expect("bind 8090");
    let done = Arc::new(AtomicBool::new(false));
    let done_server = Arc::clone(&done);

    let handle = thread::spawn(move || {
        let _ = listener.set_nonblocking(true);
        while !done_server.load(Ordering::SeqCst) {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let header =
                    "HTTP/1.1 200 OK\r\nContent-Length: 10000000\r\n\r\n{\"terrain_name\":\"mock\"}";
                let _ = stream.write_all(header.as_bytes());
                let chunk = [b'X'; 32768];
                for _ in 0..300 {
                    if stream.write_all(&chunk).is_err() {
                        break;
                    }
                }
                break;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/run_live_demo.sh"
    );
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("bash -c 'source \"{script}\"' || true"))
        .env_remove("XRTAK_URL")
        .output()
        .expect("run script");

    done.store(true, Ordering::SeqCst);
    let _ = handle.join();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        !combined.contains("[ERROR] xrtak-gateway is not reachable on :8090 or :8091"),
        "gateway probe failed to detect running gateway on :8090: {combined}"
    );
    assert!(
        combined.contains("Gateway Endpoint : http://127.0.0.1:8090"),
        "expected Gateway Endpoint : http://127.0.0.1:8090, got: {combined}"
    );
}
