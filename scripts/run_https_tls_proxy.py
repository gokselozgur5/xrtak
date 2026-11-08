#!/Library/Frameworks/Python.framework/Versions/3.11/bin/python3
"""
XR-TAK HTTPS / TLS 1.3 Reverse Proxy (:8443 -> http://127.0.0.1:8090 / :8091)
Enables Meta Quest 3 WebXR `immersive-ar` Passthrough over local LAN (`https://192.168.1.6:8443`).
Automatically generates a Subject Alternative Name (SAN) X.509 certificate for LAN & localhost.
"""

import http.client
import http.server
import os
import socket
import socketserver
import ssl
import subprocess
import sys

CERT_PATH = "/tmp/xrtak_tls_cert.pem"
KEY_PATH = "/tmp/xrtak_tls_key.pem"
CNF_PATH = "/tmp/xrtak_openssl.cnf"
OPENSSL_BIN = "/usr/bin/openssl"
UPSTREAM_HOST = "127.0.0.1"
HTTPS_PORT = int(os.environ.get("XRTAK_HTTPS_PORT", "8443"))
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INDEX_HTML_PATH = os.path.join(REPO_ROOT, "index.html")


def detect_upstream_port() -> int:
    if "XRTAK_UPSTREAM_PORT" in os.environ:
        return int(os.environ["XRTAK_UPSTREAM_PORT"])
    for p in (8090, 8091):
        try:
            conn = http.client.HTTPConnection(UPSTREAM_HOST, p, timeout=1.0)
            conn.request("GET", "/api/state")
            resp = conn.getresponse()
            body = resp.read()
            conn.close()
            if resp.status == 200 and b"terrain_name" in body:
                return p
        except Exception:
            pass
    return 8090


UPSTREAM_PORT = detect_upstream_port()


def detect_lan_ip() -> str:
    return "192.168.1.6"


def ensure_tls_cert() -> str:
    lan_ip = detect_lan_ip()
    cnf = """[req]
distinguished_name = req_distinguished_name
x509_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = 192.168.1.6
O = XR-TAK Spatial C2

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
IP.2 = 192.168.1.6
IP.3 = 192.168.1.2
"""
    with open(CNF_PATH, "w", encoding="utf-8") as f:
        f.write(cnf)
    cmd = [
        OPENSSL_BIN,
        "req",
        "-x509",
        "-newkey",
        "rsa:2048",
        "-nodes",
        "-keyout",
        KEY_PATH,
        "-out",
        CERT_PATH,
        "-days",
        "365",
        "-config",
        CNF_PATH,
        "-extensions",
        "v3_req",
    ]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return lan_ip


class ThreadingHTTPSServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


class ReverseProxyHandler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        pass

    def _proxy(self):
        clean_path = self.path.split("?", 1)[0]
        if self.command == "GET" and clean_path in ("/", "/index.html") and os.path.isfile(INDEX_HTML_PATH):
            try:
                with open(INDEX_HTML_PATH, "rb") as f:
                    data = f.read()
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.send_header("Access-Control-Allow-Private-Network", "true")
                self.end_headers()
                self.wfile.write(data)
                self.wfile.flush()
                return
            except (BrokenPipeError, ConnectionResetError):
                return
            except Exception:
                pass

        content_len = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(content_len) if content_len > 0 else None

        is_sse = self.path.startswith("/api/stream")
        conn = http.client.HTTPConnection(UPSTREAM_HOST, UPSTREAM_PORT, timeout=300 if is_sse else 12)
        try:
            fwd_headers = {
                k: v
                for k, v in self.headers.items()
                if k.lower() not in ("host", "connection", "transfer-encoding")
            }
            conn.request(self.command, self.path, body=body, headers=fwd_headers)
            resp = conn.getresponse()

            self.send_response(resp.status, resp.reason)
            is_sse = self.path.startswith("/api/stream")
            for k, v in resp.getheaders():
                kl = k.lower()
                if kl in ("transfer-encoding", "connection"):
                    continue
                self.send_header(k, v)
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Access-Control-Allow-Private-Network", "true")
            if is_sse:
                self.send_header("Connection", "close")
            self.end_headers()

            if is_sse:
                while True:
                    chunk = resp.fp.readline()
                    if not chunk:
                        break
                    self.wfile.write(chunk)
                    self.wfile.flush()
            else:
                data = resp.read()
                self.wfile.write(data)
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass
        except Exception as exc:
            try:
                msg = f'{{"error":"upstream gateway error: {exc}"}}'.encode("utf-8")
                self.send_response(502)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(msg)))
                self.end_headers()
                self.wfile.write(msg)
            except Exception:
                pass
        finally:
            conn.close()

    def do_GET(self):
        self._proxy()

    def do_POST(self):
        self._proxy()

    def do_OPTIONS(self):
        self._proxy()


def main():
    lan_ip = ensure_tls_cert()
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(certfile=CERT_PATH, keyfile=KEY_PATH)

    httpd = ThreadingHTTPSServer(("0.0.0.0", HTTPS_PORT), ReverseProxyHandler)
    httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)
    print(
        f"[xrtak-https] Quest 3 WebXR HTTPS Passthrough Gateway listening on https://{lan_ip}:{HTTPS_PORT} -> http://{UPSTREAM_HOST}:{UPSTREAM_PORT}",
        flush=True,
    )
    httpd.serve_forever()


if __name__ == "__main__":
    main()
