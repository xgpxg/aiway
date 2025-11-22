use crate::SV;
use dashmap::DashMap;

#[derive(Debug, Default)]
pub struct ResponseContext {
    pub response_ts: SV<i64>,
    /// 响应状态码
    pub status: SV<Option<u16>>,
    /// 响应头
    pub headers: DashMap<String, String>,
    /// 响应体
    pub body: SV<Vec<u8>>,
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

    pub fn set_header(&self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).map(|v| v.value().clone())
    }

    pub fn remove_header(&self, key: &str) {
        self.headers.remove(key);
    }

    pub fn set_body(&self, body: Vec<u8>) {
        self.body.set(body)
    }

    pub fn get_body(&self) -> Option<&Vec<u8>> {
        self.body.get()
    }
}
