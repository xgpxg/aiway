use crate::SV;
use bytes::Bytes;
use dashmap::DashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::error::Error;
use std::fmt::Debug;
use std::pin::Pin;
use tokio_stream::Stream;

type StreamItem = Result<Vec<u8>, Box<dyn Error + Send + Sync>>;

/// 响应上下文
#[derive(Default)]
pub struct ResponseContext {
    pub response_ts: SV<i64>,
    /// 响应状态码
    pub status: SV<Option<u16>>,
    /// 响应头
    pub headers: DashMap<String, String>,
    /// 响应体
    pub body: SV<Bytes>,
    /// 响应流
    pub stream_body: SV<Option<Pin<Box<dyn Stream<Item = StreamItem> + Send>>>>,
    /// 扩展数据
    pub state: DashMap<String, Value>,
}

impl Debug for ResponseContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseContext")
            .field("response_ts", &self.response_ts)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body", &self.body)
            //.field("stream_body", &"Stream<Item = Vec<u8>>")
            .finish()
    }
}

impl ResponseContext {
    pub fn set_response_ts(&self, ts: i64) {
        self.response_ts.set(ts);
    }

    pub fn get_response_ts(&self) -> i64 {
        *self.response_ts.get().unwrap_or(&0)
    }

    pub fn set_status(&self, status: u16) {
        self.status.set(Some(status));
    }

    pub fn get_status(&self) -> Option<u16> {
        self.status.get().and_then(|opt| *opt)
    }

    pub fn insert_header(&self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).map(|v| v.value().clone())
    }

    pub fn set_headers<H: IntoIterator<Item = (String, String)>>(&self, headers: H) {
        for (key, value) in headers {
            self.headers.insert(key, value);
        }
    }

    pub fn remove_header(&self, key: &str) {
        self.headers.remove(key);
    }

    pub fn clear_headers(&self) {
        self.headers.clear();
    }

    pub fn set_body(&self, body: Bytes) {
        self.body.set(body)
    }

    pub fn get_body(&self) -> Option<&Bytes> {
        self.body.get()
    }

    pub fn clear_body(&self) {
        self.body.set(Bytes::new());
    }

    pub fn set_stream_body(&self, body: Pin<Box<dyn Stream<Item = StreamItem> + Send>>) {
        self.stream_body.set(Some(body));
    }

    pub fn take_stream_body(&self) -> Option<Pin<Box<dyn Stream<Item = StreamItem> + Send>>> {
        self.stream_body.take().unwrap_or_default()
    }

    pub fn insert_state<T: Serialize>(&self, key: &str, value: T) {
        self.state.insert(
            key.to_string(),
            serde_json::to_value(value).expect("Failed to serialize state value"),
        );
    }

    pub fn get_state<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, serde_json::Error> {
        self.state
            .get(key)
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
    }

    pub fn is_success(&self) -> bool {
        self.status
            .get()
            .map(|s| s.unwrap_or(0) >= 200 && s.unwrap_or(0) < 300)
            .unwrap_or(false)
    }

    pub fn is_client_error(&self) -> bool {
        self.status
            .get()
            .map(|s| s.unwrap_or(0) >= 400 && s.unwrap_or(0) < 500)
            .unwrap_or(false)
    }

    pub fn is_server_error(&self) -> bool {
        self.status
            .get()
            .map(|s| s.unwrap_or(0) >= 500)
            .unwrap_or(false)
    }
}
