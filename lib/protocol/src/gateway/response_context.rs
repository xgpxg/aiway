use dashmap::DashMap;
use crate::SV;

#[derive(Debug, Default)]
pub struct ResponseContext {
    /// 响应状态码
    pub status: SV<Option<u16>>,
    /// 响应头
    pub headers: DashMap<String, String>,
    /// 响应体
    pub body: SV<Vec<u8>>,
}


impl ResponseContext {
    pub fn set_status(&self, status: u16) {
        self.status.set(Some(status));
    }

    pub fn get_status(&self) -> Option<u16> {
        self.status.get().and_then(|opt| opt.clone())
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
        self.body.get().clone()
    }
}