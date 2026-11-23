use serde::Serialize;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

pub const CORS_HEADERS: &str =
    "Access-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n";

pub struct HttpResponse {
    pub status: &'static str,
    pub content_type: Option<&'static str>,
    pub cors: bool,
    pub cache_no_cache: bool,
    pub body: Vec<u8>,
}

impl HttpResponse {
    fn new(
        status: &'static str,
        content_type: Option<&'static str>,
        cors: bool,
        body: Vec<u8>,
    ) -> Self {
        Self {
            status,
            content_type,
            cors,
            cache_no_cache: false,
            body,
        }
    }

    pub fn json_str(body: String) -> Self {
        Self::new("200 OK", Some("application/json"), true, body.into_bytes())
    }

    pub fn json(val: &impl Serialize) -> Result<Self, DynError> {
        Ok(Self::json_str(serde_json::to_string(val)?))
    }

    pub fn bad_request(err: &str) -> Self {
        let msg = serde_json::json!({ "error": err }).to_string();
        Self::new(
            "400 Bad Request",
            Some("application/json"),
            true,
            msg.into_bytes(),
        )
    }
}

pub async fn write_response(stream: &mut TcpStream, resp: &HttpResponse) -> Result<(), DynError> {
    let ct = resp
        .content_type
        .map(|c| format!("Content-Type: {c}\r\n"))
        .unwrap_or_default();
    let cors = if resp.cors { CORS_HEADERS } else { "" };
    let cache = if resp.cache_no_cache {
        "Cache-Control: no-cache\r\n"
    } else {
        ""
    };
    let header = format!(
        "HTTP/1.1 {}\r\n{ct}{cors}Content-Length: {}\r\n{cache}Connection: close\r\n\r\n",
        resp.status,
        resp.body.len()
    );
    stream.write_all(header.as_bytes()).await?;
    if !resp.body.is_empty() {
        stream.write_all(&resp.body).await?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpRequest {
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.is_empty() {
            return None;
        }
        let raw = String::from_utf8_lossy(buf);
        let (head, body) = raw
            .split_once("\r\n\r\n")
            .map(|(h, b)| (h, b.trim()))
            .unwrap_or((raw.as_ref(), ""));
        let mut lines = head.lines();
        let mut parts = lines.next()?.split_whitespace();
        let method = parts.next()?.to_string();
        let target = parts.next()?.to_string();
        let mut headers = Vec::new();
        for line in lines.take_while(|l| !l.is_empty()) {
            if let Some((k, v)) = line.split_once(':') {
                headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
            }
        }
        Some(Self {
            method,
            target,
            headers,
            body: body.to_string(),
        })
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

async fn read_http_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, DynError> {
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 8192];
    let (hdr_end, content_len) = loop {
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let cl = String::from_utf8_lossy(&buf[..pos])
                .lines()
                .find_map(|l| {
                    let (k, v) = l.split_once(':')?;
                    k.trim()
                        .eq_ignore_ascii_case("content-length")
                        .then(|| v.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            break (pos, cl);
        }
        let r = stream.read(&mut chunk).await?;
        if r == 0 {
            return Ok(buf);
        }
        buf.extend_from_slice(&chunk[..r]);
    };
    let total_needed = hdr_end + 4 + content_len;
    while buf.len() < total_needed {
        let r = stream.read(&mut chunk).await?;
        if r == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..r]);
    }
    Ok(buf)
}

pub async fn read_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, DynError> {
    match tokio::time::timeout(Duration::from_secs(10), read_http_bytes(stream)).await {
        Ok(Ok(buf)) => Ok(HttpRequest::parse(&buf)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_extracts_method_target_headers_and_trimmed_body() {
        let raw = b"POST /api/spawn?x=1 HTTP/1.1\r\nHost: 127.0.0.1:8090\r\nORIGIN:http://127.0.0.1:8090\r\n\r\n  {\"a\":1} \n";
        let req = HttpRequest::parse(raw).expect("valid request");
        assert_eq!(req.method, "POST");
        assert_eq!(req.target, "/api/spawn?x=1");
        assert_eq!(req.header("Host"), Some("127.0.0.1:8090"));
        assert_eq!(req.header("origin"), Some("http://127.0.0.1:8090"));
        assert_eq!(req.body, "{\"a\":1}");
    }

    #[test]
    fn parse_rejects_empty_or_incomplete_request_line() {
        assert!(HttpRequest::parse(b"").is_none());
        assert!(HttpRequest::parse(b"GARBAGE\r\n\r\n").is_none());
    }
}
