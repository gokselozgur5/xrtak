use std::process::Command;

const PY_DRIVER: &str = r#"
import http.client, http.server, importlib.util, json, os, shutil, signal, socket, ssl, stat, subprocess, sys, tempfile, threading
signal.alarm(5)
script = sys.argv[1]
os.environ.update({"XRTAK_UPSTREAM_PORT": "9"})
for k in ("XRTAK_TLS_DIR", "XRTAK_LAN_IP"): os.environ.pop(k, None)
spec = importlib.util.spec_from_file_location("tls_proxy", script)
mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
dk = os.path.realpath(mod.KEY_PATH)
shared = {os.path.realpath(p) for p in ("/tmp", "/var/tmp", tempfile.gettempdir()) if os.path.exists(p)}
outside_tmp = not any(dk == r or dk.startswith(r + os.sep) for r in shared)
orig_sock = socket.socket
class FakeSock:
    def __enter__(self): return self
    def __exit__(self, *a): pass
    def connect(self, a): pass
    def getsockname(self): return ("198.51.100.99", 54321)
socket.socket = lambda *a, **k: FakeSock(); detected_ip = mod.detect_lan_ip(); socket.socket = orig_sock
tdir = os.path.join(os.path.expanduser("~"), ".local", "state", "xrtak", f"tls-test-{os.getpid()}")
os.environ.update({"XRTAK_TLS_DIR": tdir, "XRTAK_LAN_IP": "198.51.100.24"})
try:
    spec.loader.exec_module(mod); lan_ip = mod.ensure_tls_cert()
    key_mode = stat.S_IMODE(os.stat(mod.KEY_PATH).st_mode)
    cert_txt = subprocess.check_output([mod.OPENSSL_BIN, "x509", "-in", mod.CERT_PATH, "-noout", "-text"], text=True)
    class Up(http.server.BaseHTTPRequestHandler):
        def log_message(self, *a): pass
        def do_GET(self):
            self.send_response(200); self.send_header("Access-Control-Allow-Origin", "*"); self.send_header("Content-Length", "2"); self.end_headers(); self.wfile.write(b"ok")
    up = http.server.HTTPServer(("127.0.0.1", 0), Up); threading.Thread(target=up.serve_forever, daemon=True).start()
    mod.UPSTREAM_PORT = up.server_port; sctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); sctx.load_cert_chain(mod.CERT_PATH, mod.KEY_PATH)
    srv = mod.ThreadingHTTPSServer(("127.0.0.1", 0), mod.ReverseProxyHandler, sctx); threading.Thread(target=srv.serve_forever, daemon=True).start()
    conn = http.client.HTTPSConnection("127.0.0.1", srv.server_port, context=ssl._create_unverified_context(), timeout=2)
    conn.request("GET", "/api/state", headers={"Connection": "close"}); resp = conn.getresponse(); resp.read(); conn.close()
    acao = [v for k, v in resp.getheaders() if k.lower() == "access-control-allow-origin"]
    srv.shutdown(); srv.server_close(); up.shutdown(); up.server_close()
finally:
    shutil.rmtree(tdir, ignore_errors=True)
print(json.dumps({"outside_tmp": outside_tmp, "detected_ip": detected_ip, "lan_ip": lan_ip, "key_mode": key_mode, "cert_ok": "198.51.100.24" in cert_txt and "192.168.1.6" not in cert_txt, "acao": len(acao)}))
"#;

#[test]
fn tls_proxy_uses_env_python_detected_host_private_key_and_single_cors_header() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/run_https_tls_proxy.py"
    );
    let first_line = std::fs::read_to_string(script)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let out = Command::new("python3")
        .args(["-c", PY_DRIVER, script])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(first_line, "#!/usr/bin/env python3");
    assert_eq!(
        (&v["outside_tmp"], &v["detected_ip"], &v["lan_ip"]),
        (
            &true.into(),
            &"198.51.100.99".into(),
            &"198.51.100.24".into()
        )
    );
    assert_eq!(
        (&v["key_mode"], &v["cert_ok"], &v["acao"]),
        (&0o600.into(), &true.into(), &1.into())
    );
}
