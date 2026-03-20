use pingora_http::ResponseHeader;

pub trait ResponseExt {
    fn is_sse(&self) -> bool;
}

impl ResponseExt for ResponseHeader {
    fn is_sse(&self) -> bool {
        self.headers
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.starts_with("text/event-stream"))
            .unwrap_or(false)
    }
}
