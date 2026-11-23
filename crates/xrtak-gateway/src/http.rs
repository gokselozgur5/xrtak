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
