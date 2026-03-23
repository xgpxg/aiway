use crate::common::header::Headers;
use http::HeaderName;
use http::response::Parts;

pub trait ResponseExt {
    fn is_sse(&self) -> bool;
    fn is_ws(&self) -> bool;
}

impl ResponseExt for Parts {
    fn is_sse(&self) -> bool {
        self.headers
            .get(Headers::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.starts_with("text/event-stream"))
            .unwrap_or(false)
    }

    fn is_ws(&self) -> bool {
        self.headers
            .get(Headers::UPGRADE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "websocket")
            .unwrap_or(false)
    }
}
